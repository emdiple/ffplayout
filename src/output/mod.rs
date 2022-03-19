use notify::{watcher, RecursiveMode, Watcher};
use std::{
    io::{prelude::*, Read},
    path::Path,
    process,
    process::{Command, Stdio},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::sleep,
    time::Duration,
};

use process_control::{ChildExt, Terminator};
use simplelog::*;
use tokio::runtime::Handle;

mod desktop;
mod stream;

use crate::input::{ingest_server, watch_folder, CurrentProgram, Source};
use crate::utils::{sec_to_time, stderr_reader, GlobalConfig, Media};

pub fn play(rt_handle: &Handle) {
    let config = GlobalConfig::global();
    let dec_settings = config.processing.clone().settings.unwrap();
    let ff_log_format = format!("level+{}", config.logging.ffmpeg_level.to_lowercase());

    let decoder_term: Arc<Mutex<Option<Terminator>>> = Arc::new(Mutex::new(None));
    let encoder_term: Arc<Mutex<Option<Terminator>>> = Arc::new(Mutex::new(None));
    let server_term: Arc<Mutex<Option<Terminator>>> = Arc::new(Mutex::new(None));
    let is_terminated: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let mut init_playlist: Option<Arc<Mutex<bool>>> = None;
    let mut live_on = false;

    let mut buffer: [u8; 65424] = [0; 65424];

    let get_source = match config.processing.clone().mode.as_str() {
        "folder" => {
            let path = config.storage.path.clone();
            if !Path::new(&path).exists() {
                error!("Folder path not exists: '{path}'");
                process::exit(0x0100);
            }

            info!("Playout in folder mode.");

            let folder_source = Source::new();
            let (sender, receiver) = channel();
            let mut watcher = watcher(sender, Duration::from_secs(2)).unwrap();

            watcher
                .watch(path.clone(), RecursiveMode::Recursive)
                .unwrap();

            debug!("Monitor folder: <b><magenta>{}</></b>", path);

            rt_handle.spawn(watch_folder(receiver, Arc::clone(&folder_source.nodes)));

            Box::new(folder_source) as Box<dyn Iterator<Item = Media>>
        }
        "playlist" => {
            info!("Playout in playlist mode");
            let program = CurrentProgram::new(rt_handle.clone(), is_terminated.clone());
            init_playlist = Some(program.init.clone());
            Box::new(program) as Box<dyn Iterator<Item = Media>>
        }
        _ => {
            error!("Process Mode not exists!");
            process::exit(0x0100);
        }
    };

    let mut enc_proc = match config.out.mode.as_str() {
        "desktop" => desktop::output(ff_log_format.clone()),
        "stream" => stream::output(ff_log_format.clone()),
        _ => panic!("Output mode doesn't exists!"),
    };

    let enc_terminator = match enc_proc.terminator() {
        Ok(proc) => Some(proc),
        Err(_) => None,
    };
    *encoder_term.lock().unwrap() = enc_terminator;

    rt_handle.spawn(stderr_reader(
        enc_proc.stderr.take().unwrap(),
        "Encoder".to_string(),
    ));

    let (ingest_sender, ingest_receiver): (Sender<[u8; 32256]>, Receiver<([u8; 32256])>) =
        channel();

    if config.ingest.enable {
        rt_handle.spawn(ingest_server(
            ff_log_format.clone(),
            ingest_sender,
            rt_handle.clone(),
            server_term.clone(),
            is_terminated.clone(),
        ));
    }

    'source_iter: for node in get_source {
        let cmd = match node.cmd {
            Some(cmd) => cmd,
            None => break,
        };

        if !node.process.unwrap() {
            continue;
        }

        info!(
            "Play for <yellow>{}</>: <b><magenta>{}</></b>",
            sec_to_time(node.out - node.seek),
            node.source
        );

        let filter = node.filter.unwrap();
        let mut dec_cmd = vec!["-hide_banner", "-nostats", "-v", ff_log_format.as_str()];

        dec_cmd.append(&mut cmd.iter().map(String::as_str).collect());

        if filter.len() > 1 {
            dec_cmd.append(&mut filter.iter().map(String::as_str).collect());
        }

        dec_cmd.append(&mut dec_settings.iter().map(String::as_str).collect());
        debug!("Decoder CMD: <bright-blue>{:?}</>", dec_cmd);

        let mut dec_proc = match Command::new("ffmpeg")
            .args(dec_cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Err(e) => {
                error!("couldn't spawn decoder process: {}", e);
                panic!("couldn't spawn decoder process: {}", e)
            }
            Ok(proc) => proc,
        };

        let dec_terminator = match dec_proc.terminator() {
            Ok(proc) => Some(proc),
            Err(_) => None,
        };
        *decoder_term.lock().unwrap() = dec_terminator;

        let mut enc_writer = enc_proc.stdin.as_ref().unwrap();
        let dec_reader = dec_proc.stdout.as_mut().unwrap();

        rt_handle.spawn(stderr_reader(
            dec_proc.stderr.take().unwrap(),
            "Decoder".to_string(),
        ));

        let mut kill_dec = true;

        loop {
            let dec_bytes_len = match dec_reader.read(&mut buffer[..]) {
                Ok(length) => length,
                Err(e) => panic!("Reading error from decoder: {:?}", e),
            };

            if let Ok(receive) = ingest_receiver.try_recv() {
                if let Err(e) = enc_writer.write_all(&receive) {
                    error!("Ingest receiver error: {:?}", e);

                    break 'source_iter;
                };

                live_on = true;

                if kill_dec {
                    if let Some(dec) = &*decoder_term.lock().unwrap() {
                        unsafe {
                            if let Ok(_) = dec.terminate() {
                                info!("Switch from {} to live ingest", config.processing.mode);
                            }
                        }
                    };

                    kill_dec = false;

                    if let Some(init) = &init_playlist {
                        *init.lock().unwrap() = true;
                    }
                }
            } else if dec_bytes_len > 0 {
                if let Err(e) = enc_writer.write(&buffer[..dec_bytes_len]) {
                    error!("Encoder write error: {:?}", e);

                    break 'source_iter;
                };
            } else {
                if live_on {
                    info!("Switch from live ingest to {}", config.processing.mode);

                    live_on = false;
                }

                break;
            }
        }

        if let Err(e) = dec_proc.wait() {
            panic!("Decoder error: {:?}", e)
        };
    }

    *is_terminated.lock().unwrap() = true;

    sleep(Duration::from_secs(1));

    if let Some(dec) = &*decoder_term.lock().unwrap() {
        unsafe {
            if let Ok(_) = dec.terminate() {
                debug!("Terminate decoder done");
            }
        }
    };

    if let Some(enc) = &*encoder_term.lock().unwrap() {
        unsafe {
            if let Ok(_) = enc.terminate() {
                debug!("Terminate encoder done");
            }
        }
    };

    if let Some(server) = &*server_term.lock().unwrap() {
        unsafe {
            if let Ok(_) = server.terminate() {
                debug!("Terminate server done");
            }
        }
    };

    info!("Playout done...");
}

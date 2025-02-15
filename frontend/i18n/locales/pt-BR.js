export default {
    ok: 'Ok',
    cancel: 'Cancelar',
    socketConnected: 'Aviso! stream conectada.',
    socketDisconnected: 'Aviso! stream desconectado.',
    alert: {
        wrongLogin: 'Dados incorretos!',
    },
    button: {
        login: 'Logar',
        home: 'Início',
        player: 'Player',
        media: 'Armazenamento',
        message: 'Legenda',
        logging: 'Registro',
        channels: 'Canais',
        configure: 'Configurar',
        logout: 'Sair',
    },
    error: {
        notFound: 'Página não encontrada',
        serverError: 'Erro do servidor interno',
    },
    input: {
        username: 'Usuário',
        password: 'Senha',
    },
    system: {
        cpu: 'CPU',
        cores: 'Núcleos',
        load: 'Carga de CPU',
        memory: 'Memória',
        swap: 'Swap',
        total: 'Total',
        usage: 'Uso',
        network: 'Rede',
        in: 'Entrada',
        out: 'Saída',
        storage: 'Armazenamento',
        device: 'Dispositivo',
        size: 'Tamanho total',
        used: 'Disponível',
    },
    control: {
        noClip: 'Nenhum clipe está sendo reproduzido',
        ingest: 'Ingestão ao Vivo',
        start: 'Iniciar Serviço do Playout',
        last: 'Ir para o último Clipe',
        stop: 'Parar Serviço do Playout',
        reset: 'Redefinir Estado do Playout',
        restart: 'Reiniciar Serviço do Playout',
        next: 'Ir para o próximo Clipe',
    },
    player: {
        start: 'Horário',
        file: 'Arquivo',
        play: 'Play',
        title: 'Título',
        duration: 'Duração',
        total: 'Total',
        in: 'Início',
        out: 'Fim',
        ad: 'Ad',
        edit: 'Editar',
        delete: 'Deletar',
        copy: 'Copiar playlist',
        loop: 'Repetir Clipes na Playlist',
        remote: 'Adicionar fonte (remota) à Playlist',
        import: 'Importar arquivo de texto/.m3u8',
        generate: 'Gerador de Playlist simples e avançado',
        reset: 'Resetar Playlist',
        save: 'Salvar Playlist',
        deletePlaylist: 'Deletar Playlist',
        unsavedProgram: 'Existe uma Programação que não está salva!',
        copyTo: 'Copiar Programação atual para',
        addEdit: 'Adicionar/Editar Fonte',
        audio: 'Áudio',
        customFilter: 'Filtro Personalizado',
        deleteFrom: 'Excluir programação de',
        deleteSuccess: 'Lista de reprodução excluída...',
        generateProgram: 'Gerar Programação',
        simple: 'Simples',
        advanced: 'Avançado',
        sorted: 'Ordenado',
        shuffle: 'Aleatório',
        shift: 'Diferença horária',
        all: 'Todos',
        addBlock: 'Adicionar bloco de tempo',
        infinitInfo: 'O playout é executado no modo infinito. Nenhuma informação baseada em tempo é possível',
        generateDone: 'Gerar lista de reprodução concluída...',
        dateYesterday: 'A hora atual é anterior à hora de início da lista de reprodução!',
        splitVideo: 'Split Video',
        cuts: 'Cuts',
    },
    media: {
        notExists: 'O armazenamento não existe!',
        create: 'Criar Pasta',
        upload: 'Enviar Arquivos',
        delete: 'Deletar',
        file: 'Arquivo',
        folder: 'Pasta',
        deleteQuestion: 'Tem certeza que deseja deletar?',
        preview: 'Visualizar',
        rename: 'Renomear Arquivo',
        newFile: 'Novo nome de arquivo',
        createFolder: 'Criar Pasta',
        foldername: 'Nome da Pasta',
        current: 'Atual',
        overall: 'Total',
        uploading: 'Enviando',
        moveError: 'Erro ao mover',
        deleteError: 'Erro ao deletar!',
        folderExists: 'A pasta já existe',
        folderCreate: 'Criação da pasta concluída...',
        folderError: 'Erro ao criar pasta',
        uploadError: 'Erro ao carregar',
        fileExists: 'O arquivo já existe!',
        recursive: 'Recursivo',
    },
    message: {
        savePreset: 'Salvar predefinição',
        newPreset: 'Nova predefinição',
        delPreset: 'Excluir predefinição',
        delText: 'Tem certeza de que deseja excluir a predefinição?',
        placeholder: 'Mensagem',
        xAxis: 'Eixo X',
        yAxis: 'Eixo Y',
        showBox: 'Mostrar caixa',
        boxColor: 'Cor da caixa',
        boxAlpha: 'Caixa Alfa',
        size: 'Tamanho',
        spacing: 'Espaçamento',
        overallAlpha: 'Alfa geral',
        fontColor: 'Cor da fonte',
        fontAlpha: 'Fonte Alfa',
        borderWidth: 'Largura da borda',
        send: 'Enviar',
        name: 'Nome',
        saveDone: 'Salvar predefinição concluída!',
        saveFailed: 'Falha ao salvar a predefinição!',
        sendDone: 'Enviando com sucesso...',
        sendFailed: 'O envio falhou...',
    },
    log: {
        download: 'Baixar arquivo de registro',
        reload: 'Recarregar',
    },
    advanced: {
        title: 'Configurações avançadas',
        decoder: 'Decoder',
        encoder: 'Encoder',
        filter: 'Filter',
        ingest: 'Ingest',
        updateSuccess: 'Configurações avançadas salva!',
        updateFailed: 'Configurações avançadas foi falhado!',
        warning: 'Aviso! Essas configurações são experimentais e destinadas apenas a usuários avançados familiarizados com o ffmpeg. Altere as configurações aqui apenas se tiver certeza do que está fazendo! As configurações podem tornar o sistema instável.',
    },
    config: {
        channel: 'Canal',
        user: 'Usuário',
        channelConf: 'Configuração do Canal',
        addChannel: 'Adicionar novo Canal',
        name: 'Nome',
        previewUrl: 'URL de Visualização',
        extensions: 'Extensões Extras',
        save: 'Salvar',
        delete: 'Deletar',
        updateChannelSuccess: 'Atualização da configuração do canal bem-sucedida!',
        updateChannelFailed: 'Falha na atualização da configuração do canal!',
        errorChannelDelete: 'O primeiro canal não pode ser deletado!',
        deleteChannelSuccess: 'Exclusão da configuração da canal bem-sucedida!',
        deleteChannelFailed: 'Falha na exclusão da configuração da canal!',
        playoutConf: 'Configuração de Playout',
        general: 'Geral',
        rpcServer: 'RPC Server',
        mail: 'Email',
        logging: 'Registro',
        processing: 'Processamento',
        ingest: 'Ingestão',
        playlist: 'Playlist',
        storage: 'Armazenamento',
        text: 'Texto',
        task: 'Tarefa',
        output: 'Saída',
        placeholderPass: 'Senha',
        help: 'Ajuda',
        generalHelp: 'Às vezes pode acontecer de um arquivo estar corrompido, mas ainda ser reproduzível. Isso pode causar um erro de streaming para todos os arquivos seguintes. A única solução nesse caso é parar o ffplayout e reiniciá-lo.',
        stopThreshold: 'O limite para o ffplayout se ele estiver fora de sincronia acima deste valor. Um número abaixo de 3 pode causar erros inesperados.',
        mailHelp: `Envie mensagens de erro para um endereço de e-mail, como clipes ausentes, formato de playlist ausente ou inválido, etc. Deixe o destinatário em branco se não precisar disso.`,
        mailInterval: 'O intervalo se refere ao número de segundos até o envio de um novo e-mail; o valor deve ser em incrementos de 10 e não inferior a 30 segundos.',
        logHelp: 'Ajuste o comportamento de log.',
        logDetect: 'Registra uma mensagem de erro se a linha de áudio estiver em silêncio por 15 segundos durante o processo de validação.',
        logIgnore: 'Ignorar strings que contenham linhas correspondentes; o formato é uma lista separada por ponto e vírgula.',
        processingHelp: 'O processamento padrão para todos os clipes garante a exclusividade.',
        processingLogoPath: 'O logotipo só é usado se o caminho existir; o caminho é relativo à pasta de armazenamento.',
        processingLogoScale: `Deixe a escala do logotipo em branco se não for necessário escalonamento. O formato é 'largura:altura', por exemplo: '100:-1' para escalonamento proporcional.`,
        processingLogoPosition: `A posição é especificada no formato 'x:y'.`,
        processingAudioTracks: 'Especifique quantas faixas de áudio devem ser processadas.',
        processingAudioIndex: 'Qual linha de áudio usar, -1 para todas.',
        processingAudioChannels: 'Defina a contagem de canais de áudio, se o áudio tiver mais canais do que estéreo.',
        processingCustomFilter: 'Adicione filtros personalizados ao processamento. As saídas de filtro devem terminar com [c_v_out] para filtros de vídeo e [c_a_out] para filtros de áudio.',
        processingOverrideFilter: 'Attention: This option overwrites all standard filters, i.e. automatic format correction no longer takes place, the command must be structured as follows: -filter_complex [0:v]fps=25,scale=1280:-1[vout];[0:a:0]volume=0.5[aout] -map [vout] -map [aout]',
        processingVTTEnable: 'VTT só pode ser usado no modo HLS e apenas se houver arquivos *.vtt com o mesmo nome do arquivo de vídeo.',
        processingVTTDummy: 'Um espaço reservado é necessário se não houver arquivo vtt.',
        ingestHelp: `Execute um servidor para um fluxo de ingestão. Este fluxo substituirá o streaming normal até que termine. Há apenas um mecanismo de autenticação simples que verifica se o nome do fluxo está correto.`,
        ingestCustomFilter: 'Aplique um filtro personalizado ao fluxo de ingestão da mesma forma que na seção de Processamento.',
        playlistHelp: 'Gerenciamento de playlist.',
        playlistDayStart: 'A que horas a playlist deve começar; deixe em branco se a playlist sempre começar do início.',
        playlistLength: 'Duração alvo da playlist; quando estiver em branco, o comprimento real não será considerado.',
        playlistInfinit: 'Reproduza infinitamente um único arquivo de playlist.',
        storageHelp: 'Configurações de armazenamento, os locais são relativos ao armazenamento do canal.',
        storageFiller: 'Use um preenchimento para reproduzir no lugar de um arquivo ausente ou preencher o tempo restante para alcançar um total de 24 horas. Pode ser um arquivo ou uma pasta com caminho relativo, e será repetido quando necessário.',
        storageExtension: 'Especifique quais arquivos procurar e usar.',
        storageShuffle: 'Escolha arquivos aleatoriamente (no modo de pasta e geração de playlist).',
        textHelp: 'Sobrepor texto em combinação com libzmq para manipulação remota de texto.',
        textFont: 'Caminho relativo ao armazenamento do canal.',
        textFromFile: 'Extração de texto a partir de um nome de arquivo.',
        textStyle: 'Defina os parâmetros drawtext, como posição, cor, etc. Postar texto pela API substituirá isso.',
        textRegex: 'Formate nomes de arquivos para extrair um título deles.',
        taskHelp: 'Execute um programa externo com um objeto de mídia fornecido. O objeto de mídia está em formato JSON e contém todas as informações sobre o clipe atual. O programa externo pode ser um script ou binário, mas deve ser executado apenas por um curto período de tempo.',
        taskPath: 'Caminho para o executável.',
        outputHelp: `A codificação final do playout, ajuste as configurações de acordo com suas necessidades. Use o modo 'stream' e ajuste o 'Parâmetro de Saída' quando quiser fazer streaming para um servidor RTMP/RTSP/SRT/... No ambiente de produção, não sirva playlists HLS com ffplayout; use Nginx ou outro servidor web!`,
        outputParam: 'Os caminhos dos segmentos e playlists HLS são relativos.',
        restartTile: 'Reiniciar Playout',
        restartText: 'Reiniciar o ffplayout para aplicar as alterações?',
        updatePlayoutSuccess: 'Sucesso na atualização da configuração do playout!',
        updatePlayoutFailed: 'Falha na atualização da configuração do playout!',
        forbiddenPlaylistPath: 'Acesso proibido: A pasta da lista de reprodução não pode ser aberta',
        noPlayoutConfig: 'Nenhuma configuração de playout encontrada!',
        publicPath: 'Public (HLS) Path',
        playlistPath: 'Playlist Path',
        storagePath: 'Storage Path',
        sharedStorage: 'O ffplayout é executado dentro de um contêiner; use a mesma raiz de armazenamento para todos os canais!',
        timezone: 'Timezone',
    },
    user: {
        title: 'Configuração de usuário',
        add: 'Adicionar usuário',
        delete: 'Deletar',
        name: 'Nome de usuário',
        mail: 'Email',
        password: 'Senha',
        newPass: 'Nova Senha',
        confirmPass: 'Confirmar Senha',
        save: 'Salvar',
        admin: 'Administrador',
        deleteNotPossible: 'Excluir o usuário atual não é possível!',
        deleteSuccess: 'Usuário deletado com sucesso!',
        deleteError: 'Erro ao deletar usuário',
        addSuccess: 'Usuário adicionado com sucesso!',
        addFailed: 'Falha ao adicionar usuário!',
        mismatch: 'Senhas não coincidem!',
        updateSuccess: 'Atualização do perfil do usuário bem-sucedida! ',
        updateFailed: 'Atualização do perfil do usuário falhou!',
    },
}

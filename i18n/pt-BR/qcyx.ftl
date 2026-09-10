cli-welcome = Iniciando o Gerenciador Bluetooth QCYx...

core-connecting = Conectando ao dispositivo...
core-connected = Conectado!
core-connected-named = Conectado a {$name}!
core-subscribed = Inscrito no canal de notificações!

error-bluetooth-adapter = Não foi possível encontrar um adaptador Bluetooth compatível.
error-device-not-found = Nenhum dispositivo QCY encontrado. Certifique-se de que estão fora da case.
error-service-not-found = Serviço GATT QCY não encontrado no dispositivo.
error-connection-dropped = Conectou, mas o dispositivo derrubou o link logo em seguida. Isso geralmente significa que ele ainda não está pareado via BLE com o Windows — vá em Configurações > Bluetooth e dispositivos e pareie por lá (isso é separado do pareamento de áudio que você já fez), depois tente de novo.
error-operation-timeout = A pilha Bluetooth parou de responder e a operação foi abandonada. Reconecte e tente de novo.

cli-anc-set = Cenário de ANC definido — modo: {$mode}, sub-cenário: {$sub_scene}, valor de ruído: {$noise_value}
cli-anc-unconfirmed = Comando de ANC enviado, mas o dispositivo não confirmou — pode não ter sido aplicado.
cli-anc-timeout = Comando de ANC enviado, mas nenhuma confirmação chegou a tempo — pode não ter sido aplicado.
cli-anc-echoed = Cenário de ANC aceito pelo dispositivo (eco recebido); aguardando confirmação final...

cli-battery-title = Status da bateria:
battery-left = Fone esquerdo
battery-right = Fone direito
battery-case = Estojo
battery-charging = carregando

cli-version-title = Informações do dispositivo:
cli-version-name-label = Nome
cli-version-name-unknown = Desconhecido
cli-version-unknown = Desconhecida

cli-balance-set = Equilíbrio do canal definido para {$value} (0 = todo à esquerda, 100 = todo à direita, 50 = centralizado).

cli-eq-set = Predefinição de equalizador aplicada.

cli-eq-custom-set = Curva personalizada de equalizador aplicada.

cli-reset-default-done = Configurações redefinidas para o padrão.
cli-factory-reset-confirm = Isso vai apagar todas as configurações do dispositivo e não pode ser desfeito. Continuar?
cli-factory-reset-cancelled = Restauração de fábrica cancelada.
cli-factory-reset-done = Restauração de fábrica enviada.
cli-rename-done = Dispositivo renomeado para "{$name}". O novo nome aparece depois de reconectar.
cli-rename-empty = O nome informado ficou vazio depois de removido espaços e caracteres de controle — nada foi enviado ao dispositivo.

cli-notification-volume-set = Volume de notificação definido.
cli-scheduled-power-off-set = Desligamento programado definido.
cli-disconnect-power-off-set = Desligamento por desconexão definido.
cli-wear-detection-set = Detecção de uso definida.
cli-game-mode-set = Modo de jogo definido.
cli-sleep-mode-set = Modo de sono definido.
cli-ldac-set = Alternância de LDAC definida.
cli-multipoint-set = Alternância de conexão dupla definida.
cli-touch-action-set = Ação de toque definida.

waiting-title = Aguardando dispositivo...
waiting-subtitle = Certifique-se de que seus fones estão por perto e fora da case.
connected-header = Conectado
error-title = Erro de Conexão
error-unknown = Erro desconhecido

gui-anc-title = Controle de Ruído
gui-anc-normal = Normal
gui-anc-transparency = Transparência
gui-anc-hint = Escolha um modo para aplicá-lo aos seus fones.
gui-anc-applied = Cenário de ANC aplicado.
gui-anc-unconfirmed = Comando de ANC enviado, mas o dispositivo não confirmou — pode não ter sido aplicado.
gui-anc-timeout = Comando de ANC enviado, mas nenhuma confirmação chegou a tempo — pode não ter sido aplicado.
gui-anc-echoed = Cenário de ANC aceito pelo dispositivo (eco recebido); aguardando confirmação final...
gui-anc-error = Falha ao definir o cenário de ANC: {$error}
gui-balance-error = Falha ao definir o equilíbrio do canal: {$error}
gui-retry = Tentar novamente

# Sidebar
app-title = QCYx
app-subtitle = Controle para fones QCY
nav-home = Início
nav-anc = Cancelamento de Ruído
nav-equalizer = Equalizador
nav-settings = Configurações
nav-about = Sobre
theme-label = Tema
badge-connecting = Conectando
badge-connected = Conectado
badge-error = Erro

# Shared
badge-coming-soon = Em breve

# Home
home-device-fallback-name = QCY HT08
home-firmware-label = Firmware:
home-firmware-unknown = Desconhecido
home-battery-title = Bateria
home-battery-left = Fone Esquerdo
home-battery-right = Fone Direito
home-battery-case = Estojo
home-battery-stale = Última leitura pode estar desatualizada
home-quick-actions-title = Ações rápidas
home-action-find-device = Localizar fones
home-action-game-mode = Modo de jogo
home-action-on = Ligado
home-action-off = Desligado
home-status-ready = Pronto
home-status-idle = Aguardando

# ANC tab
anc-title = Cancelamento de Ruído
anc-subtitle = Escolha um modo para aplicá-lo aos seus fones.
anc-noise-cancelling = Cancelamento de Ruído
anc-active = Ativo

anc-transparency-detail-title = Transparência
anc-vocal-enhancement-label = Aprimoramento vocal
anc-vocal-enhancement-desc = Prioriza vozes no som ambiente. Desligue para ajustar o nível de som ambiente manualmente.
anc-ambient-level-label = Nível do som ambiente

anc-nc-title = Cancelamento de Ruído
anc-nc-adaptive = Adaptativo
anc-nc-wind = Ruído Contra o Vento
anc-nc-indoor = Interior
anc-nc-daily-commute = Viagens Diárias
anc-nc-noisy = Barulho

anc-balance-title = Equilíbrio do Canal
anc-balance-centered = Centralizado
anc-balance-right = {$percent}% à direita
anc-balance-left = {$percent}% à esquerda
balance-reset-button = Redefinir

# Equalizer tab
eq-title = Equalizador
eq-subtitle = Ajuste fino do som dos seus fones.
eq-preset-label = Predefinição
eq-hint = Escolha uma predefinição para aplicá-la aos seus fones.
eq-preset-spatial = Som Espacial
eq-preset-default = Predefinido
eq-preset-popular = Popular
eq-preset-bass = Bass Pesados
eq-preset-rock = Rock
eq-preset-soft = Suave
eq-preset-classic = Clássico
eq-preset-applied = Predefinição do equalizador aplicada.
eq-preset-error = Falha ao definir a predefinição do equalizador: {$error}
eq-custom-title = Personalizar
eq-custom-desc = Edição por banda, de 31 Hz a 16 kHz, -8 a 8 dB.
eq-custom-reset = Redefinir
eq-custom-applied = Curva personalizada de equalizador aplicada.
eq-custom-error = Falha ao definir a curva personalizada de equalizador: {$error}

# Settings tab
settings-title = Configurações
settings-subtitle = Preferências e informações do dispositivo.
settings-device-name-label = Nome do dispositivo
settings-device-name-placeholder = QCY HT08
settings-save-button = Salvar
settings-rename-done = Dispositivo renomeado — o novo nome aparece depois de reconectar.
settings-rename-error = Falha ao renomear o dispositivo: {$error}
settings-inear-toggle-label = Detecção no ouvido
settings-inear-toggle-desc = Pausa a música ao remover um fone.
settings-inear-toggle-error = Falha ao definir a detecção de uso: {$error}
settings-game-mode-label = Modo de jogo
settings-game-mode-desc = Reduz a latência de áudio para jogos.
settings-game-mode-error = Falha ao definir o modo de jogo: {$error}
settings-sleep-mode-label = Modo de sono
settings-sleep-mode-desc = Otimiza os fones para dormir.
settings-sleep-mode-error = Falha ao definir o modo de sono: {$error}
settings-ldac-label = Codec LDAC
settings-ldac-desc = Codec de áudio Bluetooth de maior qualidade, quando suportado pela fonte.
settings-ldac-error = Falha ao definir o LDAC: {$error}
settings-multipoint-label = Conexão dupla
settings-multipoint-desc = Conecta a dois dispositivos ao mesmo tempo e alterna o áudio entre eles.
settings-multipoint-error = Falha ao definir a conexão dupla: {$error}
settings-touch-action-title = Ações de toque
settings-touch-action-desc = Define o que cada gesto de toque faz, por fone.
settings-touch-action-error = Falha ao definir a ação de toque: {$error}
settings-touch-action-left-single = Esquerdo · Toque único
settings-touch-action-right-single = Direito · Toque único
settings-touch-action-left-double = Esquerdo · Toque duplo
settings-touch-action-right-double = Direito · Toque duplo
settings-touch-action-left-triple = Esquerdo · Toque triplo
settings-touch-action-right-triple = Direito · Toque triplo
settings-touch-action-none = Sem efeito
settings-touch-action-play-pause = Reproduzir / Pausar
settings-touch-action-previous = Faixa anterior
settings-touch-action-next = Próxima faixa
settings-touch-action-voice-assistant = Assistente de voz
settings-touch-action-volume-up = Volume+
settings-touch-action-volume-down = Volume-
settings-touch-action-game-mode = Modo de jogo
settings-touch-action-anc = ANC
settings-notification-volume-title = Volume de notificação
settings-notification-volume-low = Baixo
settings-notification-volume-medium = Médio
settings-notification-volume-high = Alto
settings-notification-volume-max = Máximo
settings-notification-volume-error = Falha ao definir o volume de notificação: {$error}
settings-scheduled-poweroff-title = Desligamento programado
settings-scheduled-poweroff-desc = Desliga após um tempo definido, independente da reprodução ou conexão.
settings-scheduled-poweroff-off = Não ativar
settings-scheduled-poweroff-custom-placeholder = Personalizado (minutos)
settings-scheduled-poweroff-custom-button = Definir
settings-scheduled-poweroff-error = Falha ao definir o desligamento programado: {$error}
settings-disconnect-poweroff-title = Desconectar e desligar
settings-disconnect-poweroff-desc = Desliga após esses minutos sem conexão Bluetooth.
settings-disconnect-poweroff-never = Não desligue
settings-disconnect-poweroff-error = Falha ao definir o desligamento por desconexão: {$error}
settings-minutes-format = {$minutes} min
settings-firmware-section-title = Firmware
settings-firmware-version-label = Versão instalada
settings-firmware-check-button = Verificar atualizações
settings-reset-default-desc = Redefine todas as configurações do dispositivo para os valores padrão.
settings-reset-default-button = Redefinir para o padrão
settings-reset-default-done = Configurações redefinidas para o padrão.
settings-reset-default-error = Falha ao redefinir configurações: {$error}
settings-confirm-again = Toque de novo pra confirmar
settings-danger-title = Zona de risco
settings-factory-reset-desc = Restaura os fones para as configurações de fábrica.
settings-factory-reset-button = Restaurar configurações de fábrica
settings-factory-reset-done = Restauração de fábrica enviada.
settings-factory-reset-error = Falha na restauração de fábrica: {$error}

# About tab
about-description = QCYx é uma ferramenta não oficial, de código aberto, para controlar fones de ouvido QCY Bluetooth via BLE/GATT, feita a partir de engenharia reversa do protocolo.
about-repo-label = Repositório
about-license-label = Licença
about-protocol-title = Sobre o protocolo
about-protocol-note = O protocolo foi mapeado por captura direta de tráfego BLE do aplicativo oficial. Apenas os comandos confirmados na captura são implementados; o restante desta interface está pronto visualmente à espera da confirmação do protocolo.

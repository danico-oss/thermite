//! Interface internationalization.
//!
//! Italian strings are the reference language: at the interface points
//! the Italian string is passed to [`t`], which returns the translation in the
//! active language. If a translation is missing, Italian is shown (safe fallback).
//!
//! The supported languages are Italian, Brazilian Portuguese and English.

/// Interface language.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    It,
    PtBr,
    En,
}

impl Default for Language {
    fn default() -> Self {
        Language::It
    }
}

impl Language {
    /// All supported languages, in the order shown in the settings.
    pub const ALL: [Language; 3] = [Language::It, Language::PtBr, Language::En];

    /// Name of the language in the language itself (shown in the selector).
    pub fn label(self) -> &'static str {
        match self {
            Language::It => "Italiano",
            Language::PtBr => "Português (Brasil)",
            Language::En => "English",
        }
    }

    /// Code saved in the preferences file.
    pub fn code(self) -> &'static str {
        match self {
            Language::It => "it",
            Language::PtBr => "pt-BR",
            Language::En => "en",
        }
    }

    /// Parses a language code (tolerant of uppercase and variants).
    pub fn from_code(code: &str) -> Option<Language> {
        match code.trim().to_lowercase().as_str() {
            "it" | "ita" | "italiano" | "italian" => Some(Language::It),
            "pt" | "pt-br" | "pt_br" | "ptbr" | "portugues" | "português" | "portuguese" => {
                Some(Language::PtBr)
            }
            "en" | "eng" | "english" | "inglese" | "inglês" => Some(Language::En),
            _ => None,
        }
    }
}

/// Translates an Italian string into the active language.
pub fn t(lang: Language, it: &'static str) -> &'static str {
    match lang {
        Language::It => it,
        Language::En => lookup(EN, it).unwrap_or(it),
        Language::PtBr => lookup(PT, it).unwrap_or(it),
    }
}

/// Chooses between the singular and plural form based on the number.
pub fn plural(lang: Language, n: usize, one: &'static str, many: &'static str) -> &'static str {
    t(lang, if n == 1 { one } else { many })
}

fn lookup(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

// ---------------------------------------------------------------------------
// English
// ---------------------------------------------------------------------------

static EN: &[(&str, &str)] = &[
    // Menu and ribbon
    ("File", "File"),
    ("Visualizza", "View"),
    ("Vai", "Go"),
    ("Aiuto", "Help"),
    ("Nuovo progetto", "New project"),
    ("Apri file dati…", "Open data file…"),
    ("Ricarica dati sensori", "Reload sensor data"),
    ("Esporta grafico in PDF…", "Export chart to PDF…"),
    (
        "Mostra l'anteprima del grafico termico in un PDF A4",
        "Preview the thermal chart as an A4 PDF",
    ),
    ("Esci", "Quit"),
    ("Passa al tema chiaro", "Switch to light theme"),
    ("Passa al tema scuro", "Switch to dark theme"),
    ("Aggiorna elenco progetti", "Refresh project list"),
    ("Configurazioni…", "Settings…"),
    ("Informazioni", "About"),
    ("Informazioni su Thermite", "About Thermite"),
    // Tabs
    ("Gestione", "Management"),
    ("Ricerca", "Search"),
    ("Dati", "Data"),
    ("Grafico", "Chart"),
    ("Inserimento e modifica progetti", "Create and edit projects"),
    ("Elenco e ricerca progetti", "Project list and search"),
    ("Griglia dati sensori", "Sensor data grid"),
    ("Grafico termico", "Thermal chart"),
    // Groups
    ("Progetto", "Project"),
    ("Archivio", "Archive"),
    ("Elenco", "List"),
    ("File dati", "Data file"),
    ("Esporta", "Export"),
    ("Serie", "Series"),
    ("Vista", "View"),
    // Buttons
    ("Nuovo", "New"),
    ("Salva", "Save"),
    ("Elimina", "Delete"),
    ("Apri file", "Open file"),
    ("Aggiorna", "Refresh"),
    ("Ricarica", "Reload"),
    ("Pulisci", "Clear"),
    ("Punti", "Points"),
    ("Legenda", "Legend"),
    ("Griglia", "Grid"),
    ("Esporta PDF", "Export PDF"),
    ("Apri dati", "Open data"),
    ("Modifica", "Edit"),
    ("Cambia progetto", "Change project"),
    ("Salva progetto", "Save project"),
    ("Salva modifiche", "Save changes"),
    ("Annulla", "Cancel"),
    ("Sfoglia…", "Browse…"),
    ("Salva PDF…", "Save PDF…"),
    ("Chiudi", "Close"),
    // Tooltips
    (
        "Svuota il modulo per inserire un nuovo progetto",
        "Clear the form to enter a new project",
    ),
    (
        "Salva il progetto nel database (nuovo o modificato)",
        "Save the project to the database (new or edited)",
    ),
    (
        "Elimina il progetto attualmente in modifica",
        "Delete the project currently being edited",
    ),
    (
        "Collega un file CSV/Excel con i dati dei sensori",
        "Link a CSV/Excel file with sensor data",
    ),
    (
        "Ricarica l'elenco dei progetti dal database",
        "Reload the project list from the database",
    ),
    ("Azzera il testo di ricerca", "Clear the search text"),
    ("Ricarica l'elenco dei progetti", "Reload the project list"),
    (
        "Collega un file CSV/Excel al progetto",
        "Link a CSV/Excel file to the project",
    ),
    (
        "Rileggi il file dati del progetto attivo",
        "Re-read the active project's data file",
    ),
    (
        "Esporta il grafico termico in un documento PDF A4",
        "Export the thermal chart to an A4 PDF document",
    ),
    (
        "Mostra o nascondi i punti campionati",
        "Show or hide the sampled points",
    ),
    ("Mostra o nascondi la legenda", "Show or hide the legend"),
    (
        "Reimposta zoom e spostamento del grafico",
        "Reset chart zoom and pan",
    ),
    ("Torna alla griglia dei dati", "Back to the data grid"),
    (
        "Griglia dei dati del progetto attivo",
        "Active project's data grid",
    ),
    (
        "Grafico termico del progetto attivo",
        "Active project's thermal chart",
    ),
    (
        "Esporta il grafico termico in PDF",
        "Export the thermal chart to PDF",
    ),
    (
        "Mostra l'anteprima del grafico in un PDF A4",
        "Preview the chart as an A4 PDF",
    ),
    ("Reimposta zoom e spostamento", "Reset zoom and pan"),
    (
        "Visualizza griglia e grafico termico",
        "View the grid and thermal chart",
    ),
    ("Modifica o elimina il progetto", "Edit or delete the project"),
    // Status bar
    ("Pronto", "Ready"),
    ("progetto", "project"),
    ("progetti", "projects"),
    ("riga dati", "data row"),
    ("righe dati", "data rows"),
    ("sensore", "sensor"),
    ("sensori", "sensors"),
    ("campione", "sample"),
    ("campioni", "samples"),
    ("Progetto attivo:", "Active project:"),
    ("Progetto selezionato:", "Project selected:"),
    (
        "Nuovo progetto: compila i campi e salva.",
        "New project: fill in the fields and save.",
    ),
    ("Elenco aggiornato:", "List updated:"),
    (
        "Percorso del file dati aggiornato.",
        "Data file path updated.",
    ),
    ("Errore:", "Error:"),
    ("Dati ricaricati:", "Data reloaded:"),
    ("Tema scuro attivo.", "Dark theme active."),
    ("Tema chiaro attivo.", "Light theme active."),
    ("Ricerca azzerata.", "Search cleared."),
    ("Dati del progetto caricati.", "Project data loaded."),
    ("Dati ricaricati dal file.", "Data reloaded from file."),
    ("Modifica annullata.", "Edit cancelled."),
    ("pronto per la modifica.", "ready for editing."),
    ("aggiornato con successo!", "updated successfully!"),
    ("salvato con successo!", "saved successfully!"),
    ("eliminato.", "deleted."),
    // Management tab
    ("Modifica progetto", "Edit project"),
    ("ID univoco", "Unique ID"),
    ("ID assegnato automaticamente", "ID assigned automatically"),
    ("Nome progetto", "Project name"),
    ("Tipo di container", "Container type"),
    ("Temperature target (°C)", "Target temperatures (°C)"),
    ("File dati sensori", "Sensor data file"),
    ("Es. Camera termostatica A1", "e.g. Thermal chamber A1"),
    (
        "Es. Container refrigerato 20'",
        "e.g. Refrigerated container 20'",
    ),
    (
        "CSV, TXT oppure Excel (.xlsx / .xls)",
        "CSV, TXT or Excel (.xlsx / .xls)",
    ),
    (
        "Formato atteso: prima colonna la data, seconda l'ora, colonne successive i sensori.",
        "Expected format: first column the date, second the time, following columns the sensors.",
    ),
    ("valori numerici richiesti", "numeric values required"),
    ("la minima supera la massima", "minimum exceeds maximum"),
    ("Come si usa questa scheda", "How to use this tab"),
    (
        "1. Compila i campi e salva: il progetto finisce nel database SQLite.",
        "1. Fill in the fields and save: the project goes into the SQLite database.",
    ),
    (
        "2. Collega il file CSV/Excel con i dati dei sensori e usa la scheda Ricerca per aprire griglia e grafico.",
        "2. Link the CSV/Excel file with sensor data and use the Search tab to open the grid and chart.",
    ),
    (
        "3. Dalla scheda Ricerca puoi riaprire un progetto per modificarlo o eliminarlo.",
        "3. From the Search tab you can reopen a project to edit or delete it.",
    ),
    (
        "Il nome del progetto non può essere vuoto.",
        "The project name cannot be empty.",
    ),
    (
        "La temperatura minima deve essere un numero.",
        "The minimum temperature must be a number.",
    ),
    (
        "La temperatura massima deve essere un numero.",
        "The maximum temperature must be a number.",
    ),
    (
        "La temperatura minima non può superare la massima.",
        "The minimum temperature cannot exceed the maximum.",
    ),
    ("Errore SQLite:", "SQLite error:"),
    ("Errore delete:", "Delete error:"),
    (
        "Seleziona un progetto dalla scheda Ricerca prima di eliminarlo.",
        "Select a project in the Search tab before deleting it.",
    ),
    (
        "Nessun progetto selezionato da eliminare.",
        "No project selected to delete.",
    ),
    // Search tab
    (
        "Filtra per nome o tipo di container…",
        "Filter by name or container type…",
    ),
    ("Elenco progetti", "Project list"),
    (
        "Nessun progetto presente: creane uno dalla scheda Gestione.",
        "No projects yet: create one in the Management tab.",
    ),
    (
        "Nessun progetto corrisponde alla ricerca.",
        "No project matches the search.",
    ),
    ("Nome", "Name"),
    ("Container", "Container"),
    ("Min °C", "Min °C"),
    ("Max °C", "Max °C"),
    ("Azioni", "Actions"),
    ("di", "of"),
    // Data tab
    (
        "Nessun progetto attivo selezionato.",
        "No active project selected.",
    ),
    (
        "Apri un progetto dalla scheda Ricerca per vedere i dati.",
        "Open a project in the Search tab to see the data.",
    ),
    (
        "Il file dati è vuoto o non è stato caricato correttamente.",
        "The data file is empty or was not loaded correctly.",
    ),
    ("Valori dei sensori", "Sensor values"),
    ("Colonna", "Column"),
    // Chart tab
    (
        "Apri un progetto dalla scheda Ricerca per vedere il grafico.",
        "Open a project in the Search tab to see the chart.",
    ),
    ("soglie", "thresholds"),
    (
        "Dati insufficienti per il grafico: servono almeno le colonne Data, Ora e un sensore.",
        "Insufficient data for the chart: at least the Date, Time and one sensor columns are required.",
    ),
    (
        "Nessun valore numerico valido trovato nelle colonne dei sensori.",
        "No valid numeric values found in the sensor columns.",
    ),
    ("da", "from"),
    ("a", "to"),
    ("Max", "Max"),
    ("Min", "Min"),
    // Data loading
    (
        "Seleziona il file dati dei sensori (CSV o Excel)",
        "Select the sensor data file (CSV or Excel)",
    ),
    ("File Dati", "Data Files"),
    (
        "Nessun progetto selezionato per visualizzare i dati.",
        "No project selected to view the data.",
    ),
    (
        "Il file dati non esiste al percorso:",
        "The data file does not exist at the path:",
    ),
    (
        "Errore apertura file Excel:",
        "Error opening Excel file:",
    ),
    ("Errore apertura file:", "Error opening file:"),
    (
        "Il file dati è cambiato dal salvataggio (CRC non corrispondente):",
        "The data file has changed since it was saved (CRC mismatch):",
    ),
    (
        "Impossibile leggere il file per verificare il CRC:",
        "Cannot read the file to verify the CRC:",
    ),
    ("atteso", "expected"),
    ("trovato", "found"),
    (
        "Per accettare il nuovo file, risalva il progetto dalla scheda Gestione.",
        "To accept the new file, re-save the project from the Management tab.",
    ),
    // Preview and PDF
    (
        "Nessun progetto attivo: apri prima i dati di un progetto dalla scheda Ricerca.",
        "No active project: first open a project's data in the Search tab.",
    ),
    (
        "Nessun dato numerico da esportare: controlla il file dati del progetto.",
        "No numeric data to export: check the project's data file.",
    ),
    (
        "Anteprima PDF pronta: controlla e salva.",
        "PDF preview ready: review and save.",
    ),
    (
        "Anteprima PDF — Grafico termico",
        "PDF preview — Thermal chart",
    ),
    (
        "Apri la finestra di salvataggio e scrivi il file",
        "Open the save dialog and write the file",
    ),
    (
        "Così apparirà la pagina A4 orizzontale salvata.",
        "This is how the saved A4 landscape page will look.",
    ),
    ("Anteprima PDF chiusa.", "PDF preview closed."),
    ("Nessun dato da rappresentare", "No data to display"),
    ("Soglie", "Thresholds"),
    ("Sensori", "Sensors"),
    ("Campioni", "Samples"),
    ("Generato il", "Generated on"),
    ("File dati:", "Data file:"),
    ("Data e ora", "Date and time"),
    (
        "Nessun dato numerico da rappresentare.",
        "No numeric data to display.",
    ),
    (
        "Impossibile scrivere il file:",
        "Could not write the file:",
    ),
    (
        "Salva il grafico termico in PDF",
        "Save the thermal chart as PDF",
    ),
    ("Documento PDF", "PDF document"),
    (
        "Esportazione PDF annullata.",
        "PDF export cancelled.",
    ),
    ("Grafico esportato in PDF:", "Chart exported to PDF:"),
    ("Errore esportazione PDF:", "PDF export error:"),
    // Settings
    ("Configurazioni", "Settings"),
    ("Lingua", "Language"),
    (
        "Scegli la lingua dell'interfaccia.",
        "Choose the interface language.",
    ),
    ("Tema", "Theme"),
    (
        "Il tema scuro riduce l'affaticamento visivo.",
        "The dark theme reduces eye strain.",
    ),
    ("Chiaro", "Light"),
    ("Scuro", "Dark"),
    (
        "Le modifiche vengono salvate automaticamente.",
        "Changes are saved automatically.",
    ),
    // About
    ("Mappatura Termica di Trasporto", "Thermal Transport Mapping"),
    (
        "Applicazione desktop in Rust (egui / eframe) per gestire la mappatura termica di trasporto, l'analisi dei dati dei sensori (CSV / Excel) e la produzione di grafici e mappa 3D delle posizioni dei sensori.",
        "Desktop application in Rust (egui / eframe) to manage thermal transport mapping, analyse sensor data (CSV / Excel) and produce charts and a 3D map of sensor positions.",
    ),
    (
        "Software pubblicato sotto licenza\nGNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
        "Software published under the license\nGNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
    ),
    (
        "Thermite · Mappatura Termica di Trasporto",
        "Thermite · Thermal Transport Mapping",
    ),
    // 3D Sensors tab
    ("Sensori 3D", "3D Sensors"),
    (
        "Posizione dei sensori nel container 3D",
        "Sensor positions in the 3D container",
    ),
    (
        "Apri un progetto dalla scheda Ricerca per posizionare i sensori.",
        "Open a project in the Search tab to place the sensors.",
    ),
    ("sensore posizionato", "sensor placed"),
    ("sensori posizionati", "sensors placed"),
    ("Anteprima PDF", "PDF preview"),
    (
        "Mostra l'anteprima della mappa dei sensori in un PDF A4",
        "Preview the sensor map as an A4 PDF",
    ),
    ("Reimposta vista", "Reset view"),
    ("Reimposta", "Reset"),
    (
        "Riporta la rotazione e lo zoom iniziali",
        "Restore the initial rotation and zoom",
    ),
    ("Rimuovi tutti", "Remove all"),
    (
        "Rimuove tutti i sensori dalla mappa",
        "Removes all sensors from the map",
    ),
    ("Dimensioni container (m)", "Container size (m)"),
    ("Larghezza", "Width"),
    ("Altezza", "Height"),
    ("Profondità", "Depth"),
    ("Dimensioni", "Size"),
    ("Sensori del progetto", "Project sensors"),
    (
        "Nessun sensore trovato: apri i dati del progetto per elencarli.",
        "No sensors found: open the project data to list them.",
    ),
    ("Rimuovi questo sensore", "Remove this sensor"),
    (
        "Nascondi questo sensore dal diagramma",
        "Hide this sensor from the diagram",
    ),
    (
        "Mostra questo sensore nel diagramma",
        "Show this sensor in the diagram",
    ),
    (
        "Seleziona un sensore e clicca sul parallelepipedo per posizionarlo. Trascina un punto per spostarlo liberamente nello spazio, il fondo per ruotare, la rotella per lo zoom, il tasto destro su un punto per rimuoverlo.",
        "Select a sensor and click on the box to place it. Drag a point to move it freely in space, drag the background to rotate, use the wheel to zoom, right-click a point to remove it.",
    ),
    (
        "Seleziona un sensore dall'elenco prima di cliccare.",
        "Select a sensor from the list before clicking.",
    ),
    ("Sensore già posizionato:", "Sensor already placed:"),
    ("Sensore posizionato:", "Sensor placed:"),
    ("Sensore rimosso:", "Sensor removed:"),
    ("Mappa svuotata: sensori rimossi.", "Map cleared: sensors removed."),
    ("Vista reimpostata.", "View reset."),
    (
        "Posiziona almeno un sensore prima di esportare la mappa.",
        "Place at least one sensor before exporting the map.",
    ),
    (
        "Tutti i sensori sono nascosti: mostrane almeno uno per esportare.",
        "All sensors are hidden: show at least one to export.",
    ),
    (
        "Anteprima mappa pronta: controlla e salva.",
        "Map preview ready: review and save.",
    ),
    ("Anteprima PDF — Mappa sensori 3D", "PDF preview — 3D sensor map"),
    ("Anteprima mappa chiusa.", "Map preview closed."),
    ("Mappa sensori 3D", "3D sensor map"),
    (
        "Esporta la mappa dei sensori in un PDF A4",
        "Export the sensor map to an A4 PDF",
    ),
    ("Esporta mappa sensori in PDF…", "Export sensor map to PDF…"),
    (
        "Salva la mappa dei sensori in PDF",
        "Save the sensor map as PDF",
    ),
    ("Mappa esportata in PDF:", "Map exported to PDF:"),
];

// ---------------------------------------------------------------------------
// Brazilian Portuguese
// ---------------------------------------------------------------------------

static PT: &[(&str, &str)] = &[
    // Menu and ribbon
    ("File", "Arquivo"),
    ("Visualizza", "Exibir"),
    ("Vai", "Ir"),
    ("Aiuto", "Ajuda"),
    ("Nuovo progetto", "Novo projeto"),
    ("Apri file dati…", "Abrir arquivo de dados…"),
    ("Ricarica dati sensori", "Recarregar dados dos sensores"),
    ("Esporta grafico in PDF…", "Exportar gráfico em PDF…"),
    (
        "Mostra l'anteprima del grafico termico in un PDF A4",
        "Mostrar a pré-visualização do gráfico térmico em um PDF A4",
    ),
    ("Esci", "Sair"),
    ("Passa al tema chiaro", "Mudar para tema claro"),
    ("Passa al tema scuro", "Mudar para tema escuro"),
    ("Aggiorna elenco progetti", "Atualizar lista de projetos"),
    ("Configurazioni…", "Configurações…"),
    ("Informazioni", "Sobre"),
    ("Informazioni su Thermite", "Sobre o Thermite"),
    // Tabs
    ("Gestione", "Gerenciamento"),
    ("Ricerca", "Pesquisa"),
    ("Dati", "Dados"),
    ("Grafico", "Gráfico"),
    ("Inserimento e modifica progetti", "Inserção e edição de projetos"),
    ("Elenco e ricerca progetti", "Lista e pesquisa de projetos"),
    ("Griglia dati sensori", "Grade de dados dos sensores"),
    ("Grafico termico", "Gráfico térmico"),
    // Groups
    ("Progetto", "Projeto"),
    ("Archivio", "Repositório"),
    ("Elenco", "Lista"),
    ("File dati", "Arquivo de dados"),
    ("Esporta", "Exportar"),
    ("Serie", "Séries"),
    ("Vista", "Exibição"),
    // Buttons
    ("Nuovo", "Novo"),
    ("Salva", "Salvar"),
    ("Elimina", "Excluir"),
    ("Apri file", "Abrir arquivo"),
    ("Aggiorna", "Atualizar"),
    ("Ricarica", "Recarregar"),
    ("Pulisci", "Limpar"),
    ("Punti", "Pontos"),
    ("Legenda", "Legenda"),
    ("Griglia", "Grade"),
    ("Esporta PDF", "Exportar PDF"),
    ("Apri dati", "Abrir dados"),
    ("Modifica", "Editar"),
    ("Cambia progetto", "Trocar projeto"),
    ("Salva progetto", "Salvar projeto"),
    ("Salva modifiche", "Salvar alterações"),
    ("Annulla", "Cancelar"),
    ("Sfoglia…", "Procurar…"),
    ("Salva PDF…", "Salvar PDF…"),
    ("Chiudi", "Fechar"),
    // Tooltips
    (
        "Svuota il modulo per inserire un nuovo progetto",
        "Limpa o formulário para inserir um novo projeto",
    ),
    (
        "Salva il progetto nel database (nuovo o modificato)",
        "Salva o projeto no banco de dados (novo ou alterado)",
    ),
    (
        "Elimina il progetto attualmente in modifica",
        "Exclui o projeto em edição no momento",
    ),
    (
        "Collega un file CSV/Excel con i dati dei sensori",
        "Vincula um arquivo CSV/Excel com os dados dos sensores",
    ),
    (
        "Ricarica l'elenco dei progetti dal database",
        "Recarrega a lista de projetos do banco de dados",
    ),
    ("Azzera il testo di ricerca", "Limpa o texto da pesquisa"),
    ("Ricarica l'elenco dei progetti", "Recarrega a lista de projetos"),
    (
        "Collega un file CSV/Excel al progetto",
        "Vincula um arquivo CSV/Excel ao projeto",
    ),
    (
        "Rileggi il file dati del progetto attivo",
        "Relê o arquivo de dados do projeto ativo",
    ),
    (
        "Esporta il grafico termico in un documento PDF A4",
        "Exporta o gráfico térmico em um documento PDF A4",
    ),
    (
        "Mostra o nascondi i punti campionati",
        "Mostra ou oculta os pontos amostrados",
    ),
    ("Mostra o nascondi la legenda", "Mostra ou oculta a legenda"),
    (
        "Reimposta zoom e spostamento del grafico",
        "Redefine o zoom e o deslocamento do gráfico",
    ),
    ("Torna alla griglia dei dati", "Voltar para a grade de dados"),
    (
        "Griglia dei dati del progetto attivo",
        "Grade de dados do projeto ativo",
    ),
    (
        "Grafico termico del progetto attivo",
        "Gráfico térmico do projeto ativo",
    ),
    (
        "Esporta il grafico termico in PDF",
        "Exporta o gráfico térmico em PDF",
    ),
    (
        "Mostra l'anteprima del grafico in un PDF A4",
        "Mostra a pré-visualização do gráfico em um PDF A4",
    ),
    ("Reimposta zoom e spostamento", "Redefine o zoom e o deslocamento"),
    (
        "Visualizza griglia e grafico termico",
        "Exibe a grade e o gráfico térmico",
    ),
    ("Modifica o elimina il progetto", "Edita ou exclui o projeto"),
    // Status bar
    ("Pronto", "Pronto"),
    ("progetto", "projeto"),
    ("progetti", "projetos"),
    ("riga dati", "linha de dados"),
    ("righe dati", "linhas de dados"),
    ("sensore", "sensor"),
    ("sensori", "sensores"),
    ("campione", "amostra"),
    ("campioni", "amostras"),
    ("Progetto attivo:", "Projeto ativo:"),
    ("Progetto selezionato:", "Projeto selecionado:"),
    (
        "Nuovo progetto: compila i campi e salva.",
        "Novo projeto: preencha os campos e salve.",
    ),
    ("Elenco aggiornato:", "Lista atualizada:"),
    (
        "Percorso del file dati aggiornato.",
        "Caminho do arquivo de dados atualizado.",
    ),
    ("Errore:", "Erro:"),
    ("Dati ricaricati:", "Dados recarregados:"),
    ("Tema scuro attivo.", "Tema escuro ativo."),
    ("Tema chiaro attivo.", "Tema claro ativo."),
    ("Ricerca azzerata.", "Pesquisa redefinida."),
    ("Dati del progetto caricati.", "Dados do projeto carregados."),
    ("Dati ricaricati dal file.", "Dados recarregados do arquivo."),
    ("Modifica annullata.", "Edição cancelada."),
    ("pronto per la modifica.", "pronto para edição."),
    ("aggiornato con successo!", "atualizado com sucesso!"),
    ("salvato con successo!", "salvo com sucesso!"),
    ("eliminato.", "excluído."),
    // Management tab
    ("Modifica progetto", "Editar projeto"),
    ("ID univoco", "ID exclusivo"),
    ("ID assegnato automaticamente", "ID atribuído automaticamente"),
    ("Nome progetto", "Nome do projeto"),
    ("Tipo di container", "Tipo de contêiner"),
    ("Temperature target (°C)", "Temperaturas alvo (°C)"),
    ("File dati sensori", "Arquivo de dados dos sensores"),
    ("Es. Camera termostatica A1", "Ex.: Câmara térmica A1"),
    (
        "Es. Container refrigerato 20'",
        "Ex.: Contêiner refrigerado 20'",
    ),
    (
        "CSV, TXT oppure Excel (.xlsx / .xls)",
        "CSV, TXT ou Excel (.xlsx / .xls)",
    ),
    (
        "Formato atteso: prima colonna la data, seconda l'ora, colonne successive i sensori.",
        "Formato esperado: primeira coluna a data, segunda a hora, colunas seguintes os sensores.",
    ),
    ("valori numerici richiesti", "valores numéricos obrigatórios"),
    ("la minima supera la massima", "a mínima supera a máxima"),
    ("Come si usa questa scheda", "Como usar esta aba"),
    (
        "1. Compila i campi e salva: il progetto finisce nel database SQLite.",
        "1. Preencha os campos e salve: o projeto vai para o banco de dados SQLite.",
    ),
    (
        "2. Collega il file CSV/Excel con i dati dei sensori e usa la scheda Ricerca per aprire griglia e grafico.",
        "2. Vincule o arquivo CSV/Excel com os dados dos sensores e use a aba Pesquisa para abrir a grade e o gráfico.",
    ),
    (
        "3. Dalla scheda Ricerca puoi riaprire un progetto per modificarlo o eliminarlo.",
        "3. Na aba Pesquisa você pode reabrir um projeto para editá-lo ou excluí-lo.",
    ),
    (
        "Il nome del progetto non può essere vuoto.",
        "O nome do projeto não pode ficar vazio.",
    ),
    (
        "La temperatura minima deve essere un numero.",
        "A temperatura mínima deve ser um número.",
    ),
    (
        "La temperatura massima deve essere un numero.",
        "A temperatura máxima deve ser um número.",
    ),
    (
        "La temperatura minima non può superare la massima.",
        "A temperatura mínima não pode ser maior que a máxima.",
    ),
    ("Errore SQLite:", "Erro SQLite:"),
    ("Errore delete:", "Erro ao excluir:"),
    (
        "Seleziona un progetto dalla scheda Ricerca prima di eliminarlo.",
        "Selecione um projeto na aba Pesquisa antes de excluí-lo.",
    ),
    (
        "Nessun progetto selezionato da eliminare.",
        "Nenhum projeto selecionado para excluir.",
    ),
    // Search tab
    (
        "Filtra per nome o tipo di container…",
        "Filtra por nome ou tipo de contêiner…",
    ),
    ("Elenco progetti", "Lista de projetos"),
    (
        "Nessun progetto presente: creane uno dalla scheda Gestione.",
        "Nenhum projeto encontrado: crie um na aba Gerenciamento.",
    ),
    (
        "Nessun progetto corrisponde alla ricerca.",
        "Nenhum projeto corresponde à pesquisa.",
    ),
    ("Nome", "Nome"),
    ("Container", "Contêiner"),
    ("Min °C", "Mín °C"),
    ("Max °C", "Máx °C"),
    ("Azioni", "Ações"),
    ("di", "de"),
    // Data tab
    (
        "Nessun progetto attivo selezionato.",
        "Nenhum projeto ativo selecionado.",
    ),
    (
        "Apri un progetto dalla scheda Ricerca per vedere i dati.",
        "Abra um projeto na aba Pesquisa para ver os dados.",
    ),
    (
        "Il file dati è vuoto o non è stato caricato correttamente.",
        "O arquivo de dados está vazio ou não foi carregado corretamente.",
    ),
    ("Valori dei sensori", "Valores dos sensores"),
    ("Colonna", "Coluna"),
    // Chart tab
    (
        "Apri un progetto dalla scheda Ricerca per vedere il grafico.",
        "Abra um projeto na aba Pesquisa para ver o gráfico.",
    ),
    ("soglie", "limites"),
    (
        "Dati insufficienti per il grafico: servono almeno le colonne Data, Ora e un sensore.",
        "Dados insuficientes para o gráfico: são necessárias ao menos as colunas Data, Hora e um sensor.",
    ),
    (
        "Nessun valore numerico valido trovato nelle colonne dei sensori.",
        "Nenhum valor numérico válido encontrado nas colunas dos sensores.",
    ),
    ("da", "de"),
    ("a", "a"),
    ("Max", "Máx"),
    ("Min", "Mín"),
    // Data loading
    (
        "Seleziona il file dati dei sensori (CSV o Excel)",
        "Selecione o arquivo de dados dos sensores (CSV ou Excel)",
    ),
    ("File Dati", "Arquivos de Dados"),
    (
        "Nessun progetto selezionato per visualizzare i dati.",
        "Nenhum projeto selecionado para visualizar os dados.",
    ),
    (
        "Il file dati non esiste al percorso:",
        "O arquivo de dados não existe no caminho:",
    ),
    (
        "Errore apertura file Excel:",
        "Erro ao abrir o arquivo Excel:",
    ),
    ("Errore apertura file:", "Erro ao abrir o arquivo:"),
    (
        "Il file dati è cambiato dal salvataggio (CRC non corrispondente):",
        "O arquivo de dados foi alterado desde o salvamento (CRC não corresponde):",
    ),
    (
        "Impossibile leggere il file per verificare il CRC:",
        "Não foi possível ler o arquivo para verificar o CRC:",
    ),
    ("atteso", "esperado"),
    ("trovato", "encontrado"),
    (
        "Per accettare il nuovo file, risalva il progetto dalla scheda Gestione.",
        "Para aceitar o novo arquivo, salve novamente o projeto na aba Gestão.",
    ),
    // Preview and PDF
    (
        "Nessun progetto attivo: apri prima i dati di un progetto dalla scheda Ricerca.",
        "Nenhum projeto ativo: abra primeiro os dados de um projeto na aba Pesquisa.",
    ),
    (
        "Nessun dato numerico da esportare: controlla il file dati del progetto.",
        "Nenhum dado numérico para exportar: verifique o arquivo de dados do projeto.",
    ),
    (
        "Anteprima PDF pronta: controlla e salva.",
        "Pré-visualização do PDF pronta: confira e salve.",
    ),
    (
        "Anteprima PDF — Grafico termico",
        "Pré-visualização do PDF — Gráfico térmico",
    ),
    (
        "Apri la finestra di salvataggio e scrivi il file",
        "Abre a janela de salvamento e grava o arquivo",
    ),
    (
        "Così apparirà la pagina A4 orizzontale salvata.",
        "É assim que a página A4 paisagem salva ficará.",
    ),
    ("Anteprima PDF chiusa.", "Pré-visualização do PDF fechada."),
    ("Nessun dato da rappresentare", "Nenhum dado para representar"),
    ("Soglie", "Limites"),
    ("Sensori", "Sensores"),
    ("Campioni", "Amostras"),
    ("Generato il", "Gerado em"),
    ("File dati:", "Arquivo de dados:"),
    ("Data e ora", "Data e hora"),
    (
        "Nessun dato numerico da rappresentare.",
        "Nenhum dado numérico para representar.",
    ),
    (
        "Impossibile scrivere il file:",
        "Não foi possível gravar o arquivo:",
    ),
    (
        "Salva il grafico termico in PDF",
        "Salvar o gráfico térmico em PDF",
    ),
    ("Documento PDF", "Documento PDF"),
    (
        "Esportazione PDF annullata.",
        "Exportação em PDF cancelada.",
    ),
    ("Grafico esportato in PDF:", "Gráfico exportado em PDF:"),
    ("Errore esportazione PDF:", "Erro ao exportar PDF:"),
    // Settings
    ("Configurazioni", "Configurações"),
    ("Lingua", "Idioma"),
    (
        "Scegli la lingua dell'interfaccia.",
        "Escolha o idioma da interface.",
    ),
    ("Tema", "Tema"),
    (
        "Il tema scuro riduce l'affaticamento visivo.",
        "O tema escuro reduz o cansaço visual.",
    ),
    ("Chiaro", "Claro"),
    ("Scuro", "Escuro"),
    (
        "Le modifiche vengono salvate automaticamente.",
        "As alterações são salvas automaticamente.",
    ),
    // About
    (
        "Mappatura Termica di Trasporto",
        "Mapeamento Térmico de Transporte",
    ),
    (
        "Applicazione desktop in Rust (egui / eframe) per gestire la mappatura termica di trasporto, l'analisi dei dati dei sensori (CSV / Excel) e la produzione di grafici e mappa 3D delle posizioni dei sensori.",
        "Aplicativo desktop em Rust (egui / eframe) para gerenciar o mapeamento térmico de transporte, análise dos dados dos sensores (CSV / Excel) e produção de gráficos e mapa 3D das posições dos sensores.",
    ),
    (
        "Software pubblicato sotto licenza\nGNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
        "Software publicado sobre licença\nGNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
    ),
    (
        "Thermite · Mappatura Termica di Trasporto",
        "Thermite · Mapeamento Térmico de Transporte",
    ),
    // 3D Sensors tab
    ("Sensori 3D", "Sensores 3D"),
    (
        "Posizione dei sensori nel container 3D",
        "Posição dos sensores no contêiner 3D",
    ),
    (
        "Apri un progetto dalla scheda Ricerca per posizionare i sensori.",
        "Abra um projeto na aba Pesquisa para posicionar os sensores.",
    ),
    ("sensore posizionato", "sensor posicionado"),
    ("sensori posizionati", "sensores posicionados"),
    ("Anteprima PDF", "Pré-visualização PDF"),
    (
        "Mostra l'anteprima della mappa dei sensori in un PDF A4",
        "Mostra a pré-visualização do mapa dos sensores em um PDF A4",
    ),
    ("Reimposta vista", "Redefinir exibição"),
    ("Reimposta", "Redefinir"),
    (
        "Riporta la rotazione e lo zoom iniziali",
        "Restaura a rotação e o zoom iniciais",
    ),
    ("Rimuovi tutti", "Remover todos"),
    (
        "Rimuove tutti i sensori dalla mappa",
        "Remove todos os sensores do mapa",
    ),
    ("Dimensioni container (m)", "Dimensões do contêiner (m)"),
    ("Larghezza", "Largura"),
    ("Altezza", "Altura"),
    ("Profondità", "Profundidade"),
    ("Dimensioni", "Dimensões"),
    ("Sensori del progetto", "Sensores do projeto"),
    (
        "Nessun sensore trovato: apri i dati del progetto per elencarli.",
        "Nenhum sensor encontrado: abra os dados do projeto para listá-los.",
    ),
    ("Rimuovi questo sensore", "Remover este sensor"),
    (
        "Nascondi questo sensore dal diagramma",
        "Ocultar este sensor do diagrama",
    ),
    (
        "Mostra questo sensore nel diagramma",
        "Mostrar este sensor no diagrama",
    ),
    (
        "Seleziona un sensore e clicca sul parallelepipedo per posizionarlo. Trascina un punto per spostarlo liberamente nello spazio, il fondo per ruotare, la rotella per lo zoom, il tasto destro su un punto per rimuoverlo.",
        "Selecione um sensor e clique no paralelepípedo para posicioná-lo. Arraste um ponto para movê-lo livremente no espaço, o fundo para girar, a roda para o zoom, clique com o botão direito em um ponto para removê-lo.",
    ),
    (
        "Seleziona un sensore dall'elenco prima di cliccare.",
        "Selecione um sensor na lista antes de clicar.",
    ),
    ("Sensore già posizionato:", "Sensor já posicionado:"),
    ("Sensore posizionato:", "Sensor posicionado:"),
    ("Sensore rimosso:", "Sensor removido:"),
    (
        "Mappa svuotata: sensori rimossi.",
        "Mapa esvaziado: sensores removidos.",
    ),
    ("Vista reimpostata.", "Exibição redefinida."),
    (
        "Posiziona almeno un sensore prima di esportare la mappa.",
        "Posicione ao menos um sensor antes de exportar o mapa.",
    ),
    (
        "Tutti i sensori sono nascosti: mostrane almeno uno per esportare.",
        "Todos os sensores estão ocultos: mostre pelo menos um para exportar.",
    ),
    (
        "Anteprima mappa pronta: controlla e salva.",
        "Pré-visualização do mapa pronta: confira e salve.",
    ),
    (
        "Anteprima PDF — Mappa sensori 3D",
        "Pré-visualização do PDF — Mapa de sensores 3D",
    ),
    ("Anteprima mappa chiusa.", "Pré-visualização do mapa fechada."),
    ("Mappa sensori 3D", "Mapa de sensores 3D"),
    (
        "Esporta la mappa dei sensori in un PDF A4",
        "Exporta o mapa dos sensores em um PDF A4",
    ),
    (
        "Esporta mappa sensori in PDF…",
        "Exportar mapa de sensores em PDF…",
    ),
    (
        "Salva la mappa dei sensori in PDF",
        "Salvar o mapa dos sensores em PDF",
    ),
    ("Mappa esportata in PDF:", "Mapa exportado em PDF:"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italiano_e_identita() {
        assert_eq!(t(Language::It, "Salva"), "Salva");
    }

    #[test]
    fn traduce_in_inglese_e_portoghese() {
        assert_eq!(t(Language::En, "Salva"), "Save");
        assert_eq!(t(Language::PtBr, "Salva"), "Salvar");
    }

    #[test]
    fn chiave_sconosciuta_resta_italiana() {
        assert_eq!(t(Language::En, "chiave inesistente"), "chiave inesistente");
    }

    #[test]
    fn codici_lingua() {
        assert_eq!(Language::from_code("pt-BR"), Some(Language::PtBr));
        assert_eq!(Language::from_code("EN"), Some(Language::En));
        assert_eq!(Language::from_code("xx"), None);
        assert_eq!(Language::default(), Language::It);
    }

    #[test]
    fn plurale() {
        assert_eq!(plural(Language::En, 1, "project", "projects"), "project");
        assert_eq!(plural(Language::En, 2, "project", "projects"), "projects");
    }

    #[test]
    fn nessuna_chiave_duplicata() {
        for table in [EN, PT] {
            for (i, (k, _)) in table.iter().enumerate() {
                for (k2, _) in &table[i + 1..] {
                    assert_ne!(k, k2, "chiave duplicata nella tabella: {k}");
                }
            }
        }
    }

    #[test]
    fn stesse_chiavi_in_tutte_le_lingue() {
        for (k, _) in EN {
            assert!(
                PT.iter().any(|(k2, _)| k2 == k),
                "chiave presente in EN ma non in PT: {k}"
            );
        }
        for (k, _) in PT {
            assert!(
                EN.iter().any(|(k2, _)| k2 == k),
                "chiave presente in PT ma non in EN: {k}"
            );
        }
    }
}

//! Incorpora l'icona e i dati di versione nel file .exe di Windows,
//! compilando `assets/thermite.ico` con `rc.exe` dello Windows SDK.
//!
//! Nessuna dipendenza esterna: se lo SDK non è installato (o si compila per
//! un altro sistema) lo script salta senza far fallire la compilazione.
//!
//! L' `.ico` viene rigenerato dal test `icon::tests::scrivi_file_ico`, quindi
//! resta sincronizzato con il disegno procedurale di `src/icon.rs`.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/thermite.ico");

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = manifest.join("assets/thermite.ico");

    // Solo Windows: altrove non serve nulla.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if !ico.exists() {
        println!("cargo:warning=assets/thermite.ico assente: icona non incorporata");
        return;
    }
    let Some(rc_exe) = trova_rc() else {
        println!("cargo:warning=rc.exe dello Windows SDK non trovato: icona non incorporata");
        return;
    };

    // Percorsi con barra normale: rc.exe li accetta e non servono escape.
    let icona = ico.to_string_lossy().replace('\\', "/");
    // La versione viene letta da Cargo.toml: scriverla a mano in quattro punti
    // significherebbe rischiare che il VERSIONINFO di Windows resti indietro
    // rispetto all'eseguibile ogni volta che si incrementa il numero.
    let versione = env!("CARGO_PKG_VERSION");
    // FILEVERSION e PRODUCTVERSION vogliono quattro numeri: il quarto (build)
    // resta a zero.
    let virc = format!("{},0", versione.replace('.', ","));
    let testo = format!(
        r#"1 ICON "{icona}"

1 VERSIONINFO
FILEVERSION {virc}
PRODUCTVERSION {virc}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "FileDescription", "Thermite - Mappatura Termica di Trasporto"
            VALUE "FileVersion", "{versione}"
            VALUE "InternalName", "thermite"
            VALUE "OriginalFilename", "thermite.exe"
            VALUE "ProductName", "Thermite"
            VALUE "ProductVersion", "{versione}"
            VALUE "LegalCopyright", "GNU GENERAL PUBLIC LICENSE Version 3"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#
    );

    let rc_file = out_dir.join("thermite.rc");
    let res_file = out_dir.join("thermite.res");
    std::fs::write(&rc_file, testo).expect("scrittura thermite.rc");

    // `/fo` va attaccato al percorso: rc.exe non accetta argomento separato.
    let esito = Command::new(&rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res_file.display()))
        .arg(&rc_file)
        .output();
    let ok = match esito {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            println!(
                "cargo:warning=rc.exe non ha compilato l'icona: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            false
        }
        Err(e) => {
            println!("cargo:warning=rc.exe non eseguibile: {e}");
            false
        }
    };
    if ok {
        println!("cargo:rustc-link-arg-bins={}", res_file.display());
    }
}

/// Cerca `rc.exe` nelle cartelle dello Windows SDK installato.
fn trova_rc() -> Option<PathBuf> {
    // L'indicatore di architettura contiene parentesi: da leggere per nome.
    let program_files = std::env::var("ProgramFiles(x86)")
        .or_else(|_| std::env::var("ProgramFiles"))
        .ok()?;
    let radice = PathBuf::from(program_files)
        .join("Windows Kits")
        .join("10")
        .join("bin");
    // Le sottocartelle sono nomi di versione ("10.0.26100.0"): si ordina
    // come stringa, che per i numeri puntati equivale all'ordine crescente.
    let mut versioni: Vec<_> = std::fs::read_dir(&radice)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    versioni.sort();
    for v in versioni.into_iter().rev() {
        let p = radice.join(v).join("x64").join("rc.exe");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

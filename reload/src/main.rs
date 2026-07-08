use zbus::blocking::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    println!("Detectado: Sessão = {}, Desktop = {}", session_type, desktop);
    
    // Connect to the session D-Bus
    let connection = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Não foi possível conectar ao Session D-Bus: {}", e).into());
        }
    };

    if desktop.contains("GNOME") {
        println!("Reiniciando GNOME Shell via D-Bus...");
        // Re-exec GNOME Shell without closing windows (Alt+F2 'r' equivalence)
        let _reply = connection.call_method(
            Some("org.gnome.Shell"),
            "/org/gnome/Shell",
            Some("org.gnome.Shell"),
            "Eval",
            &("global.reexec()",),
        )?;
        println!("Sinal de recarregamento enviado com sucesso!");
    } else if desktop.contains("KDE") {
        println!("Recarregando KWin via D-Bus...");
        let _reply = connection.call_method(
            Some("org.kde.KWin"),
            "/KWin",
            Some("org.kde.KWin"),
            "reloadConfig",
            &(),
        )?;
        println!("Configurações do KWin recarregadas!");
    } else {
        println!("Sessão genérica detectada. Nenhuma API específica de reload disponível.");
    }
    
    Ok(())
}

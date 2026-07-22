fn detect_lang() -> &'static str {
    let lang = std::env::var("LC_MESSAGES")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_lowercase();
    if lang.starts_with("pt_br") || lang.starts_with("pt-br") { "pt_BR" }
    else if lang.starts_with("pt_pt") || lang.starts_with("pt-pt") || lang.starts_with("pt") { "pt_PT" }
    else if lang.starts_with("es") { "es" }
    else { "en" }
}

pub fn print_version(name: &str, version: &str) {
    let lang = detect_lang();
    println!("{name} (jeffutils) {version}");
    println!("{}", match lang {
        "pt_BR" => "Copyright (C) 2026 Jefferson Silva de Souza Rios.",
        "pt_PT" => "Copyright (C) 2026 Jefferson Silva de Souza Rios.",
        "es"    => "Copyright (C) 2026 Jefferson Silva de Souza Rios.",
        _       => "Copyright (C) 2026 Jefferson Silva de Souza Rios.",
    });
    println!("{}", match lang {
        "pt_BR" => "Contato: jeff.silvadsouza@gmail.com",
        "pt_PT" => "Contacto: jeff.silvadsouza@gmail.com",
        "es"    => "Contacto: jeff.silvadsouza@gmail.com",
        _       => "Contact: jeff.silvadsouza@gmail.com",
    });
    println!("{}", match lang {
        "pt_BR" => "Licença GPLv3+: GNU GPL versão 3 ou posterior <https://gnu.org/licenses/gpl.html>",
        "pt_PT" => "Licença GPLv3+: GNU GPL versão 3 ou posterior <https://gnu.org/licenses/gpl.html>",
        "es"    => "Licencia GPLv3+: GNU GPL versión 3 o posterior <https://gnu.org/licenses/gpl.html>",
        _       => "License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>",
    });
    println!("{}", match lang {
        "pt_BR" => "Este é um software livre: você é livre para alterá-lo e redistribuí-lo.",
        "pt_PT" => "Este é um software livre: você é livre para alterá-lo e redistribuí-lo.",
        "es"    => "Este es un software libre: usted es libre de modificarlo y redistribuirlo.",
        _       => "This is free software: you are free to change and redistribute it.",
    });
    println!("{}", match lang {
        "pt_BR" => "NÃO HÁ QUALQUER GARANTIA, na máxima extensão permitida em lei.",
        "pt_PT" => "NÃO HÁ QUALQUER GARANTIA, na máxima extensão permitida por lei.",
        "es"    => "NO HAY NINGUNA GARANTÍA, en la máxima extensión permitida por la ley.",
        _       => "THERE IS NO WARRANTY, to the extent permitted by law.",
    });
    println!();
    println!("{}", match lang {
        "pt_BR" => "Escrito por Jefferson Silva de Souza Rios.",
        "pt_PT" => "Escrito por Jefferson Silva de Souza Rios.",
        "es"    => "Escrito por Jefferson Silva de Souza Rios.",
        _       => "Written by Jefferson Silva de Souza Rios.",
    });
}

use config::Metadata;


pub fn format_deb_control(meta: &Metadata) -> Vec<u8> {
    format!(concat!(
        "Package: {name}\n",
        "Version: {version}\n",
        "Architecture: {arch}\n",
        "Maintainer: tin\n",  // TODO(tailhook)
        "Description: {short_description}\n",
        " {long_description}\n",
        ), name=meta.name, version=meta.version, arch=meta.architecture,
           short_description=meta.short_description,
           long_description=_control_multiline(&meta.long_description))
    .into_bytes()
}

fn _control_multiline(val: &String) -> String {
    val
        .replace("\n\n", "\n.\n")
        .replace("\n", "\n ")
}

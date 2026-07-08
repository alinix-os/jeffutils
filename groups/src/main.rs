fn main() {
    #[cfg(unix)]
    {
        unsafe {
            // Get the GIDs of the groups to which the current process belongs
            let ngroups = libc::getgroups(0, std::ptr::null_mut());
            if ngroups >= 0 {
                let mut groups = vec![0; ngroups as usize];
                if libc::getgroups(ngroups, groups.as_mut_ptr()) >= 0 {
                    let mut group_names = Vec::new();
                    for g in groups {
                        let gr = libc::getgrgid(g);
                        if !gr.is_null() {
                            let name = std::ffi::CStr::from_ptr((*gr).gr_name).to_string_lossy().into_owned();
                            group_names.push(name);
                        } else {
                            group_names.push(g.to_string());
                        }
                    }
                    println!("{}", group_names.join(" "));
                    return;
                }
            }
        }
    }
    println!("padrao");
}
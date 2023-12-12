pub fn windows() {
	windows_check_dep();
	windows_build();
}

fn windows_check_dep() {
	/*if !std::process::Command::new("rustup")
		.arg("run")
		.arg("x86_64-pc-windows-msvc")
		.arg("cargo")
		.arg("--version")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.expect("need to install x86_64-pc-windows-msvc")
		.success()
	{
		panic!("need to install x86_64-pc-windows-msvc");
	}*/

	if !std::process::Command::new("cargo")
		.arg("xwin")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.expect("need to install xwin")
		.success()
	{
		panic!("need to install xwin")
	}
}

fn windows_build() {}

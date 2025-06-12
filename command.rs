use std::process::Command;

const SSH_USER: &str = "ph-admin";

pub fn ssh_exec(host: &str, cmd: &str) -> Result<String, String> {
    let password = "8Mal8=49";

    let expect_script = format!(
        r#"
        spawn ssh -o StrictHostKeyChecking=no {}@{} "{}"
        expect {{
            "assword:" {{
                send "{}\r"
                expect {{
                    eof {{ exit 0 }}
                    timeout {{ exit 1 }}
                }}
            }}
            eof {{ exit 1 }}
        }}
        "#,
        SSH_USER,
        host,
        cmd,
        password
    );

    let output = Command::new("expect")
        .arg("-c")
        .arg(&expect_script)
        .output()
        .map_err(|e| format!("Expect execution failed: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Invalid UTF-8 in stdout: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("Command failed: {} (stderr: {}, stdout: {})", output.status, stderr, stdout))
    }
}

fn main() {
    match
        ssh_exec(
            "vg-ph-fon.local",
            "cat \'/Applications/Adobe InDesign CC 2017/Plug-Ins/priint.comet 4.1.6 R R25255/w2_license.lic\' | grep Expires"
        )
    {
        Ok(output) => println!("Output: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

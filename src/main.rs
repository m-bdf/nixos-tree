use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "/etc/nixos/flake.nix")]
    flake: String,
}

use serde_json::Value;
//type Object = std::collections::HashMap<String, String>;

fn run_nix(subcommand: &[&str]) -> Option<Value> {
    let output = std::process::Command::new("nix")
        .args(["--experimental-features", "nix-command flakes"])
        .args(subcommand)
        //.args(args)
        //.args(["--json", "--log-format", "internal-json"])
        .args(["--json"])
        .output()
        .unwrap();
    /*
        println!("{}", std::str::from_utf8(&output.stderr).unwrap());
        for json in std::str::from_utf8(&output.stderr).unwrap().split("@nix") {
            let object = serde_json::from_str::<Value>(json).unwrap();
            eprintln!("{}", object["msg"].as_str().unwrap());
        }
    */
    output
        .status
        .success()
        .then(|| serde_json::from_slice(&output.stdout).unwrap())
}

fn eval_expr(flake_uri: &str, config_name: &str, expr: &str) -> Option<Value> {
    let expr = format!(
        "
        let
            inherit (
                let flake = builtins.getFlake \"{flake_uri}\";
                in flake.nixosConfigurations.\"{config_name}\"
            ) config options pkgs;

            inherit (pkgs) lib;
        in
            {expr}
        "
    );

    run_nix(&["eval", "--impure", "--expr", &expr])
}

fn get_url(flake_uri: &str) -> String {
    run_nix(&["flake", "metadata", flake_uri, "--no-write-lock-file"]).unwrap()["url"]
        .as_str()
        .unwrap()
        .to_string()
}

fn main() {
    let args = Args::parse();

    let re = regex::Regex::new("^(.*)#([^#\"]*)$").unwrap();

    let (flake_uri, config_name) = if let Some(caps) = re.captures(&args.flake) {
        (get_url(&caps[1]), caps[2].to_string())
    } else {
        (
            get_url(&args.flake),
            std::fs::read_to_string("/proc/sys/kernel/hostname")
                .map_or("default".to_string(), |s| s.trim().to_string()),
        )
    };

    match eval_expr(
        &flake_uri,
        &config_name,
        "map (o: o.loc) (lib.collect lib.isOption options)",
    ) {
        Some(output) => println!("{:?}", output),
        None => println!("Error: flake {flake_uri}#{config_name} not found"),
    }

    use cursive::view::Nameable;
    use cursive::views::{LinearLayout, ListView, TextView};
    use cursive::{Cursive, CursiveExt};

    let tree_view = ListView::new().on_select(|root, path| {
        //root.call_on_name("details", |view| set_details(view, path));
    });

    let layout = LinearLayout::horizontal()
        .child(tree_view)
        .child(ListView::new().with_name("details"));

    let mut root = Cursive::new();
    root.add_layer(layout);
    root.add_global_callback('q', |s| s.quit());
    //root.run();
}

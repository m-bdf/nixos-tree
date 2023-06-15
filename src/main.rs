use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "/etc/nixos/flake.nix")]
    flake: String,
}

use cursive::view::Resizable;
use serde_json::Value;
//type Object = std::collections::HashMap<String, String>;

fn run_nix(subcommand: &[&str]) -> Option<Value> {
    let output = std::process::Command::new("nix")
        .args(["--experimental-features", "nix-command flakes"])
        .args(subcommand)
        //.args(args)
        .args(["--json", "--log-format", "internal-json", "--no-warn-dirty"])
        //.args(["--json"])
        .output()
        .unwrap();

    for json in std::str::from_utf8(&output.stderr)
        .unwrap()
        .split("@nix")
        .skip(1)
    {
        let object = serde_json::from_str::<Value>(json).unwrap();
        eprintln!("{}", object["msg"].as_str().unwrap());
    }

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
    run_nix(&["flake", "metadata", flake_uri, "--no-write-lock-file"])
        .expect(&format!("{flake_uri} not found"))["url"]
        .as_str()
        .unwrap()
        .to_string()
}

fn get_hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map_or("default".to_string(), |s| s.trim().to_string())
}

fn main() {
    let args = Args::parse();

    let re = regex::Regex::new("^(.*)#([^#\"]*)$").unwrap();
    let (flake_uri, config_name) = if let Some(caps) = re.captures(&args.flake) {
        (get_url(&caps[1]), caps[2].to_string())
    } else {
        (get_url(&args.flake), get_hostname())
    };

    let options = eval_expr(
        &flake_uri,
        &config_name,
        //"map (o: o.loc) (lib.collect lib.isOption options)",
        "lib.mapAttrsRecursiveCond (a: !lib.isOption a) (n: a: a.loc) options",
    )
    .expect(&format!("Error: flake {flake_uri}#{config_name} not found"));

    use cursive::view::Nameable;
    use cursive::views::{LinearLayout, SelectView};
    use cursive::{Cursive, CursiveExt};

    let mut tree_view = SelectView::new()
        .on_select(move |root, v: &Value| {
            root.call_on_name("children", |view: &mut SelectView<Value>| {
                view.clear();

                if v.is_array() {
                    let details = eval_expr(
                        &flake_uri,
                        &config_name,
                        &format!(
                            "
                            let
                                option = lib.getAttrFromPath (builtins.fromJSON ''{}'') options;
                            in
                                {{ inherit (option) loc description; }}
                            ",
                            v.to_string()
                        ),
                    )
                    .unwrap();

                    for (n, v) in details.as_object().unwrap() {
                        view.add_item(n, v.clone());
                    }
                } else {
                    for (n, v) in v.as_object().unwrap() {
                        view.add_item(n, v.clone());
                    }
                }
            })
            .unwrap()
        })
        /*.on_submit(move |root, v: &Value| {
            root.call_on_name("children", |view: &mut SelectView<Value>| {
                view.clear();

                if v.is_array() {
                    let details = eval_expr(
                        &flake_uri,
                        &config_name,
                        &format!(
                            "
                            let
                                option = lib.getAttrFromPath (builtins.fromJSON ''{}'') options;
                            in
                                {{ inherit (option) loc description; }}
                            ",
                            v.to_string()
                        ),
                    )
                    .unwrap();

                    for (n, v) in details.as_object().unwrap() {
                        view.add_item(n, v.clone());
                    }
                } else {
                    for (n, v) in v.as_object().unwrap() {
                        view.add_item(n, v.clone());
                    }
                }
            })
            .unwrap()
        })*/;

    for (n, v) in options.as_object().unwrap() {
        tree_view.add_item(n, v.clone());
    }

    let layout = LinearLayout::horizontal()
        .child(
            SelectView::<Value>::new()
                .with_name("parents")
                .full_screen(),
        )
        .child(tree_view.full_screen())
        .child(
            SelectView::<Value>::new()
                .with_name("children")
                .full_screen(),
        );

    let mut root = Cursive::new();
    root.add_fullscreen_layer(layout);
    root.add_global_callback('q', |s| s.quit());
    root.run();
}

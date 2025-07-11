pub(crate) enum Mode {
    Sync,
    Web,
}

fn print_help_and_exit() -> ! {
    log::error!("Usage: onemorebeer-ui --sync|--web");
    std::process::exit(1)
}

pub(crate) fn parse_mode() -> Mode {
    let mut args = std::env::args().skip(1);

    let Some(arg) = args.next() else {
        print_help_and_exit();
    };

    let mode = match arg.as_str() {
        "--sync" => Mode::Sync,
        "--web" => Mode::Web,
        _ => print_help_and_exit(),
    };

    if args.next().is_some() {
        print_help_and_exit();
    }

    mode
}

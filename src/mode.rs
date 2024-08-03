pub(crate) enum Mode {
    Sync,
    Web,
}

pub(crate) fn parse_mode() -> Result<Mode, lexopt::Error> {
    use lexopt::prelude::*;

    let mut sync = false;
    let mut web = false;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("sync") => sync = true,
            Long("web") => web = true,
            Long("help") => {
                println!("Usage: onemorebeer-ui --sync|--web");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    if sync && web {
        return Err(lexopt::Error::from(
            "Both --sync and --web provided, they are mutually exclusive",
        ));
    }

    if !sync && !web {
        return Err(lexopt::Error::from(
            "At least one of --sync or --web is required",
        ));
    }

    if sync {
        Ok(Mode::Sync)
    } else {
        Ok(Mode::Web)
    }
}

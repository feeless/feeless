from buildenv import builder
from docker import from_env
from logging import getLogger, StreamHandler, INFO
from os import getcwd

NAME = "feeless"

logger = getLogger(NAME)
logger.setLevel(INFO)
console_handler = StreamHandler()
console_handler.setLevel(INFO)
logger.addHandler(console_handler)


def main():
    builder.build()
    cmd = " && ".join([
        "cargo test",
        "cargo build",
        "cargo run --example cli -- target/Debug/feeless",
    ])
    container = from_env().containers.run(
        name=NAME,
        image="feeless/buildenv:0.1",
        volumes={
            getcwd(): {
                "bind": "/root/app/",
                "mode": "rw"
            },
        },
        working_dir="/root/app/",
        command=f"bash -c \"{cmd}\"",
        detach=True,
        auto_remove=True,
    )
    for log in container.logs(stream=True):
        logger.info(str(log, "utf-8"))


if __name__ == "__main__":
    main()

from docker import from_env
from docker.errors import BuildError
from logging import getLogger, StreamHandler, INFO
from platform import system as os_type
from sys import stdout

NAME = "builder"
TAG = "feeless/buildenv:0.1"
DOCKER_FILE = "buildenv/"

logger = getLogger(NAME)
logger.setLevel(INFO)
console_handler = StreamHandler(stream=stdout)
console_handler.setLevel(INFO)
logger.addHandler(console_handler)


def build_with_logs(client, dockerfile, buildargs, tag):
    builder_logger = getLogger(NAME)

    try:
        img, logs = client.build(
            path=dockerfile,
            buildargs=buildargs,
            tag=tag
        )
        for line in logs:
            if line.get("stream"):
                builder_logger.info(line["stream"].strip())
            elif line.get("aux"):
                builder_logger.info(line["aux"]["ID"].strip())
    except BuildError as e:
        for line in e.build_log:
            if line.get("stream"):
                builder_logger.info(line["stream"].strip())
        raise e


def build():
    builder_logger = getLogger(NAME)

    os = os_type()
    builder_logger.info("=============================================")
    builder_logger.info(f"Operating System Detected [{os}]")
    builder_logger.info("Building the build environment... Please wait")
    builder_logger.info("=============================================")

    docker_client = from_env().images

    if "Linux" in os:
        """
        in Linux, Docker user IDs are mapped host to container one-to-one by default
        the problem arises when container starts creating files into directories that are mounted to host
        this cause the host to have no permission to have write access to the created files
        it is better to create a user in the container that maps to the same user Id / group Id to the host
        """

        from os import environ, getgid, getuid

        build_with_logs(
            client=docker_client,
            dockerfile=DOCKER_FILE,
            buildargs={
                "USER_NAME": environ["USER"],
                "USER_ID": str(getuid()),
                "GROUP_ID": str(getgid()),
            },
            tag=TAG,
        )
    else:
        """
        Mac and Windows users are not running Docker natively and by utilizing transparent virtual machines (HyperKit, Hyper-V)
        virtual machines do not have the same problem as Linux since container and host userId is isolated 
        also since some user IDs used in Mac is mapped to reserved user ID in Linux, we just use root by default
        """
        build_with_logs(
            client=docker_client,
            dockerfile=DOCKER_FILE,
            buildargs=None,
            tag=TAG,
        )

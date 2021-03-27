FROM rust:1.51-slim

ENV LC_ALL=C.UTF-8

# Add custom user and group id (For Linux users, refer to buildenv/builder.py for more information)
ARG USER_NAME
ARG USER_ID
ARG GROUP_ID
RUN if [ ! -z ${USER_NAME} ] && [ ! -z ${USER_ID} ] && [ ! -z ${GROUP_ID} ]; then \
  groupadd -g ${GROUP_ID} ${USER_NAME} \
  && useradd -l -u ${USER_ID} -g ${USER_NAME} ${USER_NAME} \
  && install -d -m 0755 -o ${USER_NAME} -g ${USER_NAME} /home/${USER_NAME} \
;fi

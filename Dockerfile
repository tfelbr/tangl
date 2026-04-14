FROM rust:1.94.1

WORKDIR /usr/src/tangl
COPY . .

RUN make
RUN echo source /root/.local/share/bash-completion/completions/tangl >> ~/.bashrc

WORKDIR /usr/example
RUN mkdir .git
RUN tangl clone https://codeberg.org/tangl/construction-site-example.git
WORKDIR /usr/example/construction-site-example
RUN tangl clone --track
RUN git config --global user.email "tangl@example.com"
RUN git config --global user.name "tangl"

CMD ["bash"]
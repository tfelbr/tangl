FROM rust:1.94.1

RUN useradd -ms /bin/bash tangl

WORKDIR /root/src
COPY . .
RUN rm -rf target

RUN make
RUN cp -r /root/* /home/tangl
RUN cp -r /root/.local /home/tangl
RUN echo source /home/tangl/.local/share/bash-completion/completions/tangl >> /home/tangl/.bashrc

RUN mkdir -p /home/tangl/example/construction-site-example
RUN chown -R tangl:tangl /home/tangl
USER tangl
WORKDIR /home/tangl/example/construction-site-example

RUN git config --global user.email "tangl@example.com"
RUN git config --global user.name "tangl"

CMD ["bash"]
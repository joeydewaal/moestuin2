FROM linuxserver/ffmpeg:latest
COPY capture.sh /usr/local/bin/capture.sh
RUN chmod +x /usr/local/bin/capture.sh
ENTRYPOINT ["/usr/local/bin/capture.sh"]

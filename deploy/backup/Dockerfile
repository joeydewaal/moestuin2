FROM linuxserver/ffmpeg:latest
RUN apt-get update && apt-get install -y --no-install-recommends sqlite3 \
 && rm -rf /var/lib/apt/lists/*
COPY backup.sh /usr/local/bin/backup.sh
RUN chmod +x /usr/local/bin/backup.sh
ENTRYPOINT ["/usr/local/bin/backup.sh"]

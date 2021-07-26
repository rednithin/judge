FROM debian:10-slim

RUN apt-get update && apt-get upgrade -y

RUN apt-get install -y python3 python3-pip strace nodejs npm

CMD [ "sleep", "10" ]
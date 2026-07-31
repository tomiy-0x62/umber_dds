FROM fastdds_img:latest AS builder

SHELL ["/bin/bash", "-c"]

WORKDIR /home/ubuntu

# build ShapesDemo
RUN apt-get update && apt-get install -y qt6-base-dev
COPY refarence/ShapesDemo /home/ubuntu/ShapesDemo
WORKDIR /home/ubuntu/ShapesDemo
RUN . /opt/Fast-DDS/install/setup.bash && colcon build

FROM ubuntu:noble AS runner

WORKDIR /home
RUN apt-get update && apt-get install -y libtinyxml2-dev \
    iproute2 tmux curl netcat-openbsd qt6-base-dev build-essential cmake

USER ubuntu

# copy fastdds from fastdds_img
COPY --from=builder /opt/Fast-DDS /opt/Fast-DDS
# copy ShapesDemo from fastdds_img
COPY --from=builder /home/ubuntu/ShapesDemo /home/ubuntu/ShapesDemo

WORKDIR /home/ubuntu



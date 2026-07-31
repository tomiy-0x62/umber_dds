FROM fastdds_img:latest AS fast_builder

WORKDIR /home
RUN mkdir hw_workspace
SHELL ["/bin/bash", "-c"]
# build DDSHelloWorld
RUN --mount=type=bind,src=./dds-analysis-docker/workspace_DDSHelloWorld,target=./hw_workspace mkdir hw_build && cd hw_build \
    && . /opt/Fast-DDS/install/setup.bash && cmake ../hw_workspace && cmake --build .

# build ShapesDemo
RUN apt-get update && apt-get install -y qt6-base-dev
COPY ./ShapesDemo /home/ShapesDemo
WORKDIR /home/ShapesDemo
RUN . /opt/Fast-DDS/install/setup.bash && colcon build

FROM cyclonedds_img:latest AS cyclone_builder

WORKDIR /home
# build shapes_demo_cyclonedds
RUN --mount=type=bind,src=./dds-analysis-docker/cyclone_shapesdemo,target=./cyclone_shapesdemo mkdir cyclone_build && cd cyclone_build \
   && cmake ../cyclone_shapesdemo && cmake --build .

FROM ubuntu:noble AS runner

RUN apt-get update && apt-get install -y libtinyxml2-dev libtinyxml2-10 \
     qt6-base-dev iproute2 tmux netcat-openbsd \
     libxcb-cursor0 libxkbcommon-x11-0 libxcb-icccm4 \
     libxcb-image0 libxcb-keysyms1 libxcb-randr0 \
     libxcb-render-util0 libxcb-shape0 libxcb-sync1 \
     libxcb-xfixes0 libxcb-xinerama0
     # libssl-dev qtbase5-dev iproute2 tmux python3 curl locales netcat python3-pip

# copy fastdds from fastdds_img
COPY --from=fast_builder /opt/Fast-DDS /opt/Fast-DDS
# copy DDSHelloWorld from fastdds_img
COPY --from=fast_builder /home/hw_build /home/ubuntu/DDSHelloWorld
# copy ShapesDemo from fastdds_img
COPY --from=fast_builder /home/ShapesDemo /home/ubuntu/ShapesDemo
# copy cyclonedds from cyclonedds_img
COPY --from=cyclone_builder /opt/cyclonedds /opt/cyclonedds
# copy cyclonedds from cyclone_shapedemo
COPY --from=cyclone_builder /home/cyclone_build /home/ubuntu/cyclone_shapesdemo

WORKDIR /home/ubuntu

RUN mkdir -p umber_dds/target

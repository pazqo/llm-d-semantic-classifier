FROM registry.access.redhat.com/ubi9/nodejs-18:latest AS build
USER root
RUN command -v yarn || npm i -g yarn

ADD . /usr/src/app
WORKDIR /usr/src/app
RUN yarn install && yarn build

FROM registry.access.redhat.com/ubi9/nginx-122:latest

COPY --from=build /usr/src/app/dist /opt/app-root/src
COPY nginx.conf /etc/nginx/nginx.conf
USER 1001

CMD ["nginx", "-g", "daemon off;"]

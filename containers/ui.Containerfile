FROM docker.io/library/node:22.20-alpine AS build

WORKDIR /source
COPY thought-khoral-workspace-ui/package.json thought-khoral-workspace-ui/package-lock.json ./
RUN npm ci
COPY thought-khoral-workspace-ui/index.html thought-khoral-workspace-ui/tsconfig.json thought-khoral-workspace-ui/vite.config.ts ./
COPY thought-khoral-workspace-ui/src/ src/
RUN npm run build

FROM docker.io/nginxinc/nginx-unprivileged:1.29.2-alpine

USER 0
COPY --from=build /source/dist/ /usr/share/nginx/html/
COPY thought-khoral-platform/ui/nginx.conf /etc/nginx/conf.d/default.conf
COPY thought-khoral-platform/ui/thought-khoral-bootstrap.js /usr/share/nginx/html/thought-khoral-bootstrap.js
RUN app_module="$(sed -n 's/.*src="\([^\"]*\/assets\/index-[^\"]*\.js\)".*/\1/p' /usr/share/nginx/html/index.html)" \
    && test -n "$app_module" \
    && sed -i "s#<script type=\"module\"[^>]*></script>#<meta name=\"thought-khoral-app-module\" content=\"$app_module\"><script type=\"module\" src=\"/thought-khoral-bootstrap.js\"></script>#" /usr/share/nginx/html/index.html

USER 101:0
EXPOSE 8080

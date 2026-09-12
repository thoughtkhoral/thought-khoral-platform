FROM docker.io/library/node:22.20-alpine AS build

WORKDIR /source
COPY n2n-workspace-ui/package.json n2n-workspace-ui/package-lock.json ./
RUN npm ci
COPY n2n-workspace-ui/index.html n2n-workspace-ui/tsconfig.json n2n-workspace-ui/vite.config.ts ./
COPY n2n-workspace-ui/src/ src/
RUN npm run build

FROM docker.io/nginxinc/nginx-unprivileged:1.29.2-alpine

USER 0
COPY --from=build /source/dist/ /usr/share/nginx/html/
COPY n2n-platform/ui/nginx.conf /etc/nginx/conf.d/default.conf
COPY n2n-platform/ui/n2n-bootstrap.js /usr/share/nginx/html/n2n-bootstrap.js
RUN app_module="$(sed -n 's/.*src="\([^\"]*\/assets\/index-[^\"]*\.js\)".*/\1/p' /usr/share/nginx/html/index.html)" \
    && test -n "$app_module" \
    && sed -i "s#<script type=\"module\"[^>]*></script>#<meta name=\"n2n-app-module\" content=\"$app_module\"><script type=\"module\" src=\"/n2n-bootstrap.js\"></script>#" /usr/share/nginx/html/index.html

USER 101:0
EXPOSE 8080

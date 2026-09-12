FROM quay.io/keycloak/keycloak:26.4.7

ENV KC_DB=postgres \
    KC_HEALTH_ENABLED=true
RUN /opt/keycloak/bin/kc.sh build

COPY --chown=1000:0 thought-khoral-platform/keycloak/thought-khoral-dev-realm.json /opt/keycloak/data/import/thought-khoral-realm.json

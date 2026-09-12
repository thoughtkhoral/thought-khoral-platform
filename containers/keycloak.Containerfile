FROM quay.io/keycloak/keycloak:26.4.7

ENV KC_DB=postgres \
    KC_HEALTH_ENABLED=true
RUN /opt/keycloak/bin/kc.sh build

COPY --chown=1000:0 n2n-platform/keycloak/n2n-dev-realm.json /opt/keycloak/data/import/n2n-realm.json

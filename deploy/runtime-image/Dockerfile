FROM gcr.io/distroless/static-debian13:nonroot@sha256:e2e927ec666bae08560abb3c55d0659eceabb657f56b6782ab500a9fc7f555e3
ARG SOURCE_SHA
ARG RELEASE_VERSION
ARG CANDIDATE_ID
ARG METADATA_SHA256
LABEL org.opencontainers.image.source="https://github.com/apeiraco/konobangu" \
      org.opencontainers.image.revision="${SOURCE_SHA}" \
      org.opencontainers.image.version="${RELEASE_VERSION}" \
      org.opencontainers.image.licenses="MIT" \
      io.apeiraco.konobangu.candidate-id="${CANDIDATE_ID}" \
      io.apeiraco.konobangu.release-metadata-sha256="${METADATA_SHA256}"
ENV STORAGE__DATA_DIR=/data
WORKDIR /app
COPY --chown=65532:65532 --chmod=0755 recorder-cli /app/recorder-cli
COPY --chown=65532:65532 webui /app/webui
COPY --chown=65532:65532 licenses /app/licenses
COPY --chown=65532:65532 data /data
COPY --chown=65532:65532 release.json /app/release.json
COPY --chown=65532:65532 recorder.config.toml /config/recorder.config.toml
USER 65532:65532
EXPOSE 5001
VOLUME ["/data"]
ENTRYPOINT ["/app/recorder-cli"]
CMD ["--environment", "production", "--config-file", "/config/recorder.config.toml", "--working-dir", "/app"]

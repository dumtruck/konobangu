# Version, provenance, deterministic assembly and publication state utilities.
[positional-arguments]
release *args:
    node scripts/dev-cli.mts release {{recipe_args}}

# Docker is the native executor; image selection and collision checks are release utilities.
release-image kind context tags source version candidate metadata:
    docker buildx build --push --platform {{if kind == "runtime" { "linux/amd64" } else if kind == "fixture" { "linux/amd64,linux/arm64" } else { error("Use runtime or fixture") }}} --provenance=false --tag {{replace(tags, ",", " --tag ")}} --build-arg SOURCE_SHA={{source}} --build-arg RELEASE_VERSION={{version}} --build-arg CANDIDATE_ID={{candidate}} --build-arg METADATA_SHA256={{metadata}} --label org.opencontainers.image.source=https://github.com/apeiraco/konobangu --label org.opencontainers.image.revision={{source}} --label org.opencontainers.image.version={{version}} --label org.opencontainers.image.licenses=MIT --label io.apeiraco.konobangu.candidate-id={{candidate}} --label io.apeiraco.konobangu.release-metadata-sha256={{metadata}} "{{context}}"

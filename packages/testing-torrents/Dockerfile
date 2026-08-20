FROM node:24.15.0-bookworm-slim@sha256:4e6b70dd6cbfc88c8157ba19aa3d9f9cce6ba4703576d55459e45efcbc9c5f5d AS nodebt
RUN npm install -g pnpm@12.4.1
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

FROM nodebt AS deps

# Install build tools required by node-gyp for native modules (bufferutil, utf-8-validate, utp-native)
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    make \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/workspace
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml /app/
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile --store-dir /pnpm/store

FROM deps AS app

COPY main.ts /app/

EXPOSE 6080
EXPOSE 6081
EXPOSE 6082

CMD [ "node", "main.ts" ]

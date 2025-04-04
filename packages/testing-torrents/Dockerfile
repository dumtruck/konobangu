FROM node:23-slim AS nodebt
RUN npm install -g pnpm
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

FROM nodebt AS deps

RUN mkdir -p /app/workspace
WORKDIR /app
COPY package.json /app/
RUN pnpm approve-builds utf-8-validate node-datachannel utp-native
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --no-frozen-lockfile

FROM deps AS app

COPY main.ts /app/

CMD [ "npm", "start" ]

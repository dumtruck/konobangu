FROM node:23-slim AS nodebt

FROM nodebt AS deps

RUN mkdir -p /app/workspace
WORKDIR /app
COPY package.json /app/
RUN npm install

FROM deps AS app

COPY main.ts /app/

CMD [ "npm", "start" ]
# Build stage
FROM node:22-alpine AS builder
WORKDIR /app

ARG VITE_API_URL
ARG VITE_WS_URL
ENV VITE_API_URL=$VITE_API_URL
ENV VITE_WS_URL=$VITE_WS_URL

COPY package.json package-lock.json* ./
RUN npm ci

COPY . .
RUN npm run build

# Runtime stage
FROM node:22-alpine
WORKDIR /app

COPY --from=builder /app/build build/
COPY --from=builder /app/package.json .
COPY --from=builder /app/node_modules node_modules/

ENV PORT=${CLIENT_PORT:-3000}
ENV ORIGIN=${CLIENT_ORIGIN:-http://localhost:3000}
EXPOSE 3000

CMD ["node", "build"]

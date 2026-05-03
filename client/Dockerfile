# Build stage
FROM node:22-alpine AS builder
WORKDIR /app

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

ENV PORT=3000
ENV ORIGIN=http://localhost:3000
EXPOSE 3000

CMD ["node", "build"]

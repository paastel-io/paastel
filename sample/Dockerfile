FROM nginx:1.29.3-alpine

# Remove o config padrão
RUN rm /etc/nginx/conf.d/default.conf

# Copia sua configuração customizada
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copia os arquivos estáticos
COPY static /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

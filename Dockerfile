FROM nginx:alpine
LABEL authors="Bitechular Innovations"
RUN rm -rf /usr/share/nginx/html/*
COPY pkg /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

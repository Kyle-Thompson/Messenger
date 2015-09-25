C = gcc -Wall -std=c11 -o

all:	client server

client:	client.c common.h
	$(C) client client.c wrappers.c

server:	server.c common.h
	$(C) server server.c wrappers.c

clean:	
	rm -f client server

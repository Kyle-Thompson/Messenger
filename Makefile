C = gcc -Wall -std=c11 -o

all:	client server

client:	client.c errorChecking.c common.h
	$(C) client client.c errorChecking.c

server:	server.c errorChecking.c common.h
	$(C) server server.c errorChecking.c

clean:	
	rm -f client server

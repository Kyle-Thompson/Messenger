// TODO: See if all of these are even necessary
#include <sys/socket.h>
#include <netinet/in.h>
#include <sys/wait.h>
#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netdb.h>

/*for getting file size using stat()*/
#include <sys/stat.h>
 
/*for sendfile()*/
#include <sys/sendfile.h>
 
/*for O_RDONLY*/
#include <fcntl.h>

#define BUFFER 1024
#define PORT 5000
#define MAXPENDING 3

void ls  (char* command, int client_socket);
void get (char* command, int client_socket);
void put (char* command, int client_socket);
void cd  (char* command, int client_socket);
void mkdr(char* command, int client_socket);
void err (char* command, int client_socket);
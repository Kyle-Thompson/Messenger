#include "header.h"

#include <errno.h>
#include <string.h>
#include <sys/types.h>

/*for getting file size using stat()*/
#include <sys/stat.h>
 
/*for sendfile()*/
#include <sys/sendfile.h>
 
/*for O_RDONLY*/
#include <fcntl.h>


void ls  (char* command, int client_socket);
void get (char* command, int client_socket);
void put (char* command, int client_socket);
void cd  (char* command, int client_socket);
void mkdr(char* command, int client_socket);
void err (char* command, int client_socket);

int main() {
	int server_socket = 0, client_socket = 0;

	struct sockaddr_in serv_addr;

	//char sendBuff[BUFFER+1]; // Is this needed right here?
	char recvBuff[BUFFER+1];
	char command[6];

	char request[6][6] = {"ls", "get", "put", "cd", "mkdir", "err"};
	int (*request_handler[6]) (char* command, int client_socket);
	request_handler[0] = ls;
	request_handler[1] = get;
	request_handler[2] = put;
	request_handler[3] = cd;
	request_handler[4] = mkdr;
	request_handler[5] = err;

	if ((server_socket = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
		printf("socket error\n");
		return -1;
	}

	memset(&serv_addr, '0', sizeof(serv_addr));
	//memset(sendBuff, '0', sizeof(sendBuff));
	memset(recvBuff, '0', sizeof(recvBuff));

	serv_addr.sin_family = AF_INET;    
	serv_addr.sin_addr.s_addr = htonl(INADDR_ANY); 
	serv_addr.sin_port = htons(PORT);

	if (bind(server_socket, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
		printf("bind error\n");
		return -1;
	}

	if (listen(server_socket, MAXPENDING) < 0) {
		printf("listen error\n");
		return -1;
	}

	if (client_socket = accept(server_socket, (struct sockaddr*)NULL ,NULL) < 0) {
		printf("accept error\n");
		return -1
	}
  

	while (strncmp(recvBuff, "exit", 4) != 0) {
		if(recv(client_socket, recvBuff, BUFFER, 0) < 0)
			printf("Error: Receive\nErrno: %d\n", errno);
		recvBuff[BUFFER] = '\n';

		sscanf(recvBuff, "%s", command);
		command[5] = '\n';

		if (strcmp(command, "exit") == 0) break;

		// Handle request
		for (int i = 0; i < 6; ++i)
			if (strcmp(command, request[i]) == 0 || i == 5)
				request_handler[i](recvBuff, client_socket);
	}

	close(client_socket);
	close(server_socket); 

	return 0;    
}

	// while (strncmp(sendBuff, "exit", 4) != 0) {
	// 	  fputs("(Server) Enter a message: ", stdout);
	// 	  fgets(sendBuff, sizeof(sendBuff), stdin);

	// 	  if(send(client_socket, sendBuff, sizeof(sendBuff), 0) < 0)
	// 		  printf("Error: Send\nErrno: %d\n", errno);
	// }
	// close(client_socket);
	// exit(0);

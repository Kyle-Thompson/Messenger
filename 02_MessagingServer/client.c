#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netdb.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <arpa/inet.h>
 
int main(void)
{
  int sockfd = 0,n = 0;
  char recvBuff[1024];
  char message[1024] = "";
  struct sockaddr_in serv_addr;
 
  memset(recvBuff, '0' ,sizeof(recvBuff));
  if((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
    printf("\n Error : Could not create socket \n");
    return 1;
  }
 
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(5000);
  serv_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
 
  if(connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
    printf("\n Error : Connect Failed \n");
    return 1;
  }
 
  while(strncmp(message, "exit", 4) != 0) {
    fputs("Enter a message: ", stdout);
    fgets(message, sizeof(message), stdin);

    if(send(sockfd, message, sizeof(message), 0) < 0) {
      printf("Error: Send");
      printf("Errno: %d\n", errno);
    }

    if( (n = recv(sockfd, recvBuff, sizeof(recvBuff)-1, 0)) < 0) {
      printf("Error: Recieve");
      printf("Errno: %d\n", errno);
    }
    recvBuff[n] = 0;

    if(fputs(recvBuff, stdout) == EOF)
      printf("\nError: fputs");
    printf("\n");
  }
 
  if( n < 0) {
    printf("\n Read Error \n");
  }
 
  return 0;
}


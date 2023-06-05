# mini-health-check

Sends an HTTP(s) request to given command line parameters, exits successfully if response code is 2XX. For HTTPs requests certificate validation is turned off and out of scope, TLS1.2 and TLS1.3 is supported alongside algorithms rustls implements.
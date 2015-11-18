# sync-clipboard
Application which will synchronise updates to clipboard between connected computers.

When you copy a text in to a clipbord on pc-1 it will be copied in to a clipboard on pc-2 and vice versa. 
It is working only for text data so far (I don't need any other sort of data so far). 

It is working between mac and windows 10. 
Haven't tested other systems.

Usage: 
sync-clipboard --port 12345 --local youripaddress --outside the-other-ip
-p -o -l args can be used instead of long ones.

If it fails to connect to the other pc, it will start the server. The other pc then will connect to the first one.




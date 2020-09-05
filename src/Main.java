import java.io.*;
import java.net.*;
import java.util.*;

/**
 * This class represents the main starting point of Kak-Inspector.
 */
public class Main {
    static long kak_session = 0;
    
    public static void main(String[] args) {
        //If this is a client command
        if (args.length > 1 && args[0].equals("-c")) {
            handleClient(args);
        }
        //If this is meant to be the server instance
        else if (args.length > 1 && args[0].equals("-s")) {
            handleServer(args);
        }
    }

    /**
     * Sends a command back to the Kakoune session that spawned us.
     * @param command The command to execute.
     */
    static void kakCommand(String command) {
        try {
            System.out.println(command);
            ProcessBuilder builder = new ProcessBuilder();
            List<String> commands = new ArrayList<String>();
            commands.add("/bin/sh");
            commands.add("-c");
            commands.add("echo " + command + " | kak -p " + kak_session);
            builder.command(commands);
            builder.inheritIO();
            System.out.println(builder.command());

            Process process = builder.start();

            //Process process = Runtime.getRuntime().exec("printf '" + command + "' | kak -p " + kak_session);
            //We need to capture any output of the process so it doesn't block
            BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
            String line = "";
            while ((line = reader.readLine()) != null) {
                System.out.println(line);
            }
        } catch (IOException ioe) { ioe.printStackTrace();}
    }

    /**
     * Handles the client side of the Kak-Inspector application by sending a command to the server.
     */
    static void handleClient(String[] args) {
        //Hardcoded server port 8080 (TODO: make this configurable.)
        int port = 8080;

        try (Socket sock = new Socket("localhost", port)) {
            //Construct the command to send to the server.
            String[] command = new String[args.length - 1];
            for (int i = 0; i < command.length; ++i) {
                command[i] = args[i + 1];
            }

            //Send the command to the server.
            OutputStream output = sock.getOutputStream();
            PrintWriter writer = new PrintWriter(output, true);
            writer.println(String.join(" ", command));
            
        } catch (UnknownHostException ex) {
            System.out.println("Server not found: " + ex.getMessage());
        } catch (IOException ex) {
            System.out.println("I/O error: " + ex.getMessage());
        }
    }

    /**
     * Handles the server side of the Kak-Inspector application by opening a socket and listening for commands.
     */
    static void handleServer(String[] args) {
        //Get the session that called us
        try {
            kak_session = Integer.parseInt(args[1]);
        } catch (NumberFormatException nfe) {}

        kakCommand("set-option global debug_running true");
        
        //Hardcoded server port 8080 (TODO: make this configurable.)
        int port = 8080;
        try (ServerSocket sock = new ServerSocket(port)) {
            while (true) {
                Socket client = sock.accept();
                InputStream input = client.getInputStream();
                BufferedReader reader = new BufferedReader(new InputStreamReader(input));
                String[] command = reader.readLine().split(" ");
                //If this is the quit command
                if (command[0].equals("quit")) {
                    kakCommand("set-option global debug_running false");
                    sock.close();
                    break;
                }
                //Otherwise, do stuff (TODO: implement.)
                else {
                }
            }
        } catch (IOException ioe) {
            System.err.println("Server exception: " + ioe.getMessage());
            ioe.printStackTrace();
        }
    }
}

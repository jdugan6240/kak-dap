import java.io.*;
import java.nio.charset.*;

public class DAP {
    /**
     * This method gets the command to invoke the debug adapter.
     *
     * @return The command to run to invoke the debug adapter.
     */
    static String getCmd() {
        //Command is hardcoded for now.
        //TODO: return command based on config.
        return "node /home/jdugan/.vscode-oss/extensions/webfreak.debug-0.25.0/out/src/lldb.js";
    }

    static class EndOfStream extends RuntimeException {
        static final long serialVersionUID = 1;
    }

    /**
     * This method reads a single char from the given input stream.
     *
     * @return The char read
     */
    private static char read(InputStream stream) {
        try {
            //Read the character
            var c = stream.read();
            //If the character is nonsense, then the stream was closed.
            if (c == -1) {
                throw new EndOfStream();
            }
            //Return the read character
            return (char)c;
        } catch (IOException ioe) {
            throw new EndOfStream();
        }
    }

    private static String readHeader(InputStream client) {
        var line = new StringBuilder();
        //Constantly read new characters from the stream
        for (var next = read(client); true; next = read(client)) {
            //Add the character to our string
            line.append(next);
            //If we have the \r\n\r\n delimiter, then stop
            if (line.toString().endsWith("\r\n\r\n"))
                break;
        }
        //Return the resulting string
        return line.toString();
    }

    private static int parseHeader(String header) {
        var contentHeader = "Content-Length: ";
        //If the header starts with the correct string
        if (header.startsWith(contentHeader)) {
            var tail = header.substring(contentHeader.length());
            tail = tail.trim(); //Remove the newline characters
            //Extract the length of the string
            var length = Integer.parseInt(tail);
            return length;
        }
        return -1;
    }

    private static String readMessage(InputStream client, int length) {
        //Eat whitespace
        var next = read(client);
        while (Character.isWhitespace(next))
            next = read(client);

        //Construct message string
        var result = new StringBuilder();
        var i = 0;
        while (true) {
            result.append(next);
            ++i;
            if (i == length) break;
            next = read(client);
        }
        return result.toString();
    }

    static String nextMessage(InputStream client) {
        //Read the header
        var line = readHeader(client);
        //Grab the length of the message
        var contentLength = parseHeader(line);
        //Read the message itself
        return readMessage(client, contentLength);
    }

    static void writeMessage(OutputStream out, String message) {
        //Output messages in UTF-8 format
        var messageBytes = message.getBytes(StandardCharsets.UTF_8);
        var header = String.format("Content-Length: %d\r\n\r\n", messageBytes.length);
        var headerBytes = header.getBytes(StandardCharsets.UTF_8);
        try {
            //Print the header first
            out.write(headerBytes);
            //Then the message
            out.write(messageBytes);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    public static void connect() {

    }

}

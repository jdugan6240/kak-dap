import com.google.gson.*;

import java.io.*;
import java.nio.charset.*;
import java.util.concurrent.*;

/**
 * This class handles communication between the Kak-Inspector
 * instance and the debug adapter.
 */
public class DAP {
    public static final Gson gson = new Gson(); 
    /**
     * This method gets the command to invoke the debug adapter.
     *
     * @return The command to run to invoke the debug adapter.
     */
    static String getCmd() {
        //Command is hardcoded for now.
        //TODO: return command based on config.
        return "node ~/.vscode-oss/extensions/webfreak.debug-0.25.0/out/src/lldb.js";
    }

    static class EndOfStream extends RuntimeException {
        static final long serialVersionUID = 1;
    }

    /**
     * This method reads a single char from the given input stream.
     * @param in The stream to read
     * @return The char read
     */
    private static char read(InputStream in) {
        try {
            //Read the character
            var c = in.read();
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

    /**
     * This method reads the header of a message from the given input stream.
     * @param in The stream to read
     * @return The header read
     */
    private static String readHeader(InputStream in) {
        var line = new StringBuilder();
        //Constantly read new characters from the stream
        for (var next = read(in); true; next = read(in)) {
            //Add the character to our string
            line.append(next);
            //If we have the \r\n\r\n delimiter, then stop
            if (line.toString().endsWith("\r\n\r\n"))
                break;
        }
        //Return the resulting string
        return line.toString();
    }

    /**
     * This method parses the header to get the length of the following message.
     * @param header The header to parse
     * @return The length of the following message
     */
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

    /**
     * This method reads a message of the given length from the given input stream.
     * @param in The stream to read
     * @param length The length of the message to read
     * @return The resulting message
     */
    private static String readMessage(InputStream in, int length) {
        //Eat whitespace
        var next = read(in);
        while (Character.isWhitespace(next))
            next = read(in);

        //Construct message string
        var result = new StringBuilder();
        var i = 0;
        while (true) {
            result.append(next);
            ++i;
            if (i == length) break;
            next = read(in);
        }
        return result.toString();
    }

    /**
     * Convenience method that reads an entire message from the given input
     * stream, header and all, returning the message's contents.
     * @param in The stream to read
     * @return The resulting message contents
     */
    static String nextMessage(InputStream in) {
        //Read the header
        var line = readHeader(in);
        //Grab the length of the message
        var contentLength = parseHeader(line);
        //Read the message itself
        return readMessage(in, contentLength);
    }

    /**
     * This method writes the given message to the given output stream.
     * @param out The stream to write to
     * @param message The message to send
     */
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

    /**
     * This method converts a message string into a Message object that can be easily parsed.
     * @param message The message to convert
     * @return The converted message
     */
    static Message parseMessage(String message) {
        return gson.fromJson(message, Message.class);
    }

    /**
     * This method is used to begin communication with the debug adapter, using
     * the given input and output streams to facilitate this communication.
     * @param in The input stream with which to read the adapter's stdout
     * @param out The output stream with which to write to the adapter's stdin
     */
    public static void connect(InputStream in, OutputStream out) {
        var pending = new ArrayBlockingQueue<Message>(10);

        //Run the read loop in a separate thread
        class ReaderLoop implements Runnable {
            public void run() {
                while (true) {
                    try {
                        var token = nextMessage(in);
                        var message = parseMessage(token);
                        pending.put(message);
                    } catch (Exception e) {}
                }
            }
        }
        Thread reader = new Thread(new ReaderLoop(), "reader");
        reader.setDaemon(true);
        reader.start();

        var endOfStream = new Message();

        //Process messages obtained
        while (true) {
            Message r;
            try {
                r = pending.poll(200, TimeUnit.MILLISECONDS);
            } catch (Exception e) {
                continue;
            }
            //If the input stream has been closed, exit
            if (r == endOfStream)
                break;
            //If poll failed, loop again
            if (r == null)
                continue;
            //Otherwise, process the new message
            switch (r.method) {

            }
        }

    }
   
}

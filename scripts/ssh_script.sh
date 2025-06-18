#!/usr/bin/env expect

# Check for required arguments
if {$argc != 4} {
    send_user "Usage: $argv0 <user> <pass> <host> <command>\n"
    exit 1
}

# Set variables from command line arguments
set user [lindex $argv 0]
set pass [lindex $argv 1]
set host [lindex $argv 2]
set command [lindex $argv 3]

# Set timeout for the overall connection
set timeout 30
log_user 1

# Attempt counter
set auth_attempts 0

# Start SSH command
spawn ssh -F /dev/null -o ConnectTimeout=30 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o PreferredAuthentications=password -o PubkeyAuthentication=no $user@$host "$command"

# Handle expected outputs
expect {
    -re {[Pp]assword:[\s]*$} {
        send_user "Attempting authentication...\n"
        send -- "$pass\r"
        exp_continue
    }
    -re {(yes/no|fingerprint|continue connecting)} {
        send "yes\r"
        exp_continue
    }
    "Permission denied" {
        if {$auth_attempts < 3} {
            # On permission denied, increment attempts and wait for next password prompt
            incr auth_attempts
            send_user "Authentication failed, attempt $auth_attempts of 3\n"
            exp_continue
        } else {
            puts "Permission denied after multiple attempts"
            exit 1
        }
    }
    "Connection closed" {
        puts "Connection closed"
        exit 1
    }
    "account is locked" {
        puts "Account is locked"
        exit 1
    }
    timeout {
        puts "Connection timed out"
        exit 1
    }
    eof {
        # Expect script finishes when SSH exits
        puts "SSH connection terminated unexpectedly or completed"
        exit
    }
}

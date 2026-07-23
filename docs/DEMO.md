# OnionGate demo script

## Isolated app circuits

1. Connect using the Maximum Isolation preset.
2. Add two signed applications under Apps.
3. Show their bundle IDs and distinct Tor exit status.
4. Rotate one application's circuit.
5. Stop Tor and show Session Guard suspend the selected apps.
6. Reconnect and show only OnionGate-suspended processes resume.

## Private localhost onion

1. Run a development server on `127.0.0.1:3000`.
2. Create a private Onion Lab project.
3. Show the generated v3 hostname, QR, and one-time credential.
4. Run Test & audit and inspect listener scope, publication, latency, HTTP
   response, and security headers.
5. Destroy the project and show that the ephemeral key is gone.

## Leak and recovery report

1. Run Verify while protected.
2. Explain that public addresses are compared in memory but not retained.
3. Force-quit a development build after enabling TUN/firewall state.
4. Reopen OnionGate and run Emergency Restore.
5. Export the redacted JSON verification report.

Record demos on a clean VM with test applications and non-sensitive traffic.

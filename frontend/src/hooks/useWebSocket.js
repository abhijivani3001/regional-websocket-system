import { useState, useCallback, useRef } from 'react';

export const useWebSocket = (url, userId, token) => {
  const [isConnected, setIsConnected] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [messages, setMessages] = useState([]);
  const [latency, setLatency] = useState(0);
  const [error, setError] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('Disconnected');
  const ws = useRef(null);
  const lastPingRef = useRef(0);

  const connect = useCallback(() => {
    if (!url) return;

    // Prevent multiple connections
    if (ws.current?.readyState === WebSocket.OPEN) return;
    if (ws.current) ws.current.close();

    setConnectionStatus('Connecting...');

    try {
      const fullUrl = `${url}?userId=${encodeURIComponent(userId)}&token=${encodeURIComponent(token)}`;
      ws.current = new WebSocket(fullUrl);

      ws.current.onopen = () => {
        setIsConnected(true);
        setConnectionStatus('Connected');
        setError(null);

        // Send Auth message immediately after connection
        ws.current.send(
          JSON.stringify({
            event: 'auth',
            user_id: userId,
            token: token,
          }),
        );
      };

      ws.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === 'pong') {
            setLatency(Date.now() - lastPingRef.current);
          } else if (data.type === 'auth_success') {
            setIsAuthenticated(true);
            setConnectionStatus('Authenticated');
          } else if (data.type === 'auth_failure') {
            setIsAuthenticated(false);
            setError(data.reason || 'Authentication failed');
            setConnectionStatus('Auth Failed');
          } else if (data.type === 'message') {
            setMessages((prev) => [...prev, data]);
          } else if (data.type === 'error') {
            console.error('Server error:', data.message);
          }
        } catch (e) {
          console.error('Failed to parse message', e);
        }
      };

      ws.current.onclose = () => {
        setIsConnected(false);
        setIsAuthenticated(false);
        setConnectionStatus('Disconnected');
      };

      ws.current.onerror = () => {
        setError('WebSocket connection failed');
        setConnectionStatus('Error');
        setIsConnected(false);
        setIsAuthenticated(false);
      };
    } catch (e) {
      setError(e.message);
      setConnectionStatus('Error');
    }
  }, [url, userId, token]);

  const disconnect = useCallback(() => {
    if (ws.current) {
      ws.current.close();
      ws.current = null;
    }
    setIsConnected(false);
    setIsAuthenticated(false);
    setConnectionStatus('Disconnected');
  }, []);

  const sendMessage = useCallback(
    (content, recipient) => {
      if (ws.current && ws.current.readyState === WebSocket.OPEN) {
        const message = {
          event: 'message',
          to: recipient,
          content: content,
        };
        ws.current.send(JSON.stringify(message));

        setMessages((prev) => [...prev, { ...message, from: userId, timestamp: Date.now() }]);
      }
    },
    [userId],
  );

  const sendPing = useCallback(() => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      lastPingRef.current = Date.now();
      ws.current.send(JSON.stringify({ event: 'ping' }));
    }
  }, []);

  return {
    isConnected,
    isAuthenticated,
    messages,
    connectionStatus,
    error,
    latency,
    connect,
    disconnect,
    sendMessage,
    sendPing,
  };
};

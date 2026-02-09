import React, { useState, useEffect } from 'react';
import RegionSelector from './components/RegionSelector.jsx';
import LoginForm from './components/LoginForm.jsx';
import ChatInterface from './components/ChatInterface.jsx';
import { useWebSocket } from './hooks/useWebSocket.js';

function App() {
  const [step, setStep] = useState('region'); // 'region', 'login', 'chat'
  const [selectedRegion, setSelectedRegion] = useState(null);
  const [userId, setUserId] = useState('');
  const [token, setToken] = useState('');
  const [wsUrl, setWsUrl] = useState('');

  const {
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
  } = useWebSocket(wsUrl, userId, token);

  // Auto-connect when credentials are set
  useEffect(() => {
    if (wsUrl && userId && token && step === 'chat') {
      connect();
    }
  }, [wsUrl, userId, token, step, connect]);

  const handleRegionSelect = (region) => {
    setSelectedRegion(region);
    setStep('login');
  };

  const handleLogin = (newUserId, newToken) => {
    const url = `ws://localhost:${selectedRegion.port}/ws`;
    setUserId(newUserId);
    setToken(newToken);
    setWsUrl(url);
    setStep('chat');
  };

  const handleDisconnect = () => {
    disconnect();
    setStep('region');
    setSelectedRegion(null);
    setUserId('');
    setToken('');
    setWsUrl('');
  };

  const handleBack = () => {
    setStep('region');
    setSelectedRegion(null);
  };

  return (
    <div className='min-h-screen flex items-center justify-center p-4'>
      {step === 'region' && <RegionSelector selectedRegion={selectedRegion} onRegionSelect={handleRegionSelect} />}

      {step === 'login' && <LoginForm region={selectedRegion} onLogin={handleLogin} onBack={handleBack} />}

      {step === 'chat' && (
        <ChatInterface
          region={selectedRegion}
          userId={userId}
          messages={messages}
          connectionStatus={connectionStatus}
          latency={latency}
          isAuthenticated={isAuthenticated}
          onSendMessage={sendMessage}
          onDisconnect={handleDisconnect}
          onPing={sendPing}
        />
      )}

      {/* Error notification */}
      {error && (
        <div className='fixed bottom-4 right-4 bg-red-500 text-white px-6 py-3 rounded-lg shadow-lg animate-slide-in'>
          <p className='font-medium'>Error</p>
          <p className='text-sm'>{error}</p>
        </div>
      )}
    </div>
  );
}

export default App;

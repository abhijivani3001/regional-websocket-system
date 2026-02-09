import React, { useState, useEffect, useRef } from 'react';
import { Send, LogOut, Activity, Wifi } from 'lucide-react';

export default function ChatInterface({
  region,
  userId,
  messages,
  connectionStatus,
  latency,
  isAuthenticated,
  onSendMessage,
  onDisconnect,
  onPing,
}) {
  const [input, setInput] = useState('');
  const [recipient, setRecipient] = useState('');
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = (e) => {
    e.preventDefault();
    if (input.trim() && recipient.trim()) {
      onSendMessage(input, recipient);
      setInput('');
    }
  };

  console.log('ChatInterface rendered with props:', {
    region,
    userId,
    messages,
    connectionStatus,
    latency,
    isAuthenticated,
  });

  return (
    <div className='bg-white rounded-xl shadow-2xl w-full max-w-4xl h-[80vh] flex flex-col overflow-hidden'>
      {/* Header */}
      <div className='bg-gray-50 p-4 border-b flex justify-between items-center'>
        <div className='flex items-center space-x-4'>
          <div className='bg-blue-100 p-2 rounded-lg'>
            <Wifi className='w-5 h-5 text-blue-600' />
          </div>
          <div>
            <h3 className='font-bold text-gray-800'>{region.name}</h3>
            <div className='flex items-center text-xs space-x-2'>
              <span className={`flex items-center ${isAuthenticated ? 'text-green-600' : 'text-red-500'}`}>
                <span className={`w-2 h-2 rounded-full mr-1 ${isAuthenticated ? 'bg-green-500' : 'bg-red-500'}`}></span>
                {connectionStatus}
              </span>
              <span className='text-gray-400'>|</span>
              <span className='text-gray-500 flex items-center'>
                <Activity className='w-3 h-3 mr-1' /> {latency}ms
              </span>
            </div>
          </div>
        </div>
        <div className='flex items-center space-x-2'>
          <button
            onClick={onPing}
            className='px-3 py-1 text-xs bg-gray-200 hover:bg-gray-300 rounded text-gray-700 transition-colors'
          >
            Ping
          </button>
          <button onClick={onDisconnect} className='p-2 text-gray-500 hover:text-red-500 transition-colors'>
            <LogOut className='w-5 h-5' />
          </button>
        </div>
      </div>

      {/* Messages Area */}
      <div className='flex-1 overflow-y-auto p-4 space-y-4 bg-gray-50/50'>
        {messages.map((msg, idx) => (
          <div key={idx} className={`flex ${msg.from === userId ? 'justify-end' : 'justify-start'}`}>
            <div
              className={`max-w-[70%] rounded-2xl px-4 py-2 ${
                msg.from === userId
                  ? 'bg-blue-600 text-white rounded-br-none'
                  : 'bg-white border border-gray-200 rounded-bl-none'
              }`}
            >
              <div className='text-xs opacity-70 mb-1'>{msg.from}</div>
              <p>{msg.content}</p>
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <form onSubmit={handleSend} className='p-4 bg-white border-t flex gap-2'>
        <input
          type='text'
          value={recipient}
          onChange={(e) => setRecipient(e.target.value)}
          placeholder='To User ID'
          className='w-32 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none'
        />
        <input
          type='text'
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder='Type a message...'
          className='flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none'
        />
        <button
          type='submit'
          disabled={!isAuthenticated}
          className='bg-blue-600 text-white p-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors'
        >
          <Send className='w-5 h-5' />
        </button>
      </form>
    </div>
  );
}

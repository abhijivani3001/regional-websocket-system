import React, { useState } from 'react';
import { User, ArrowLeft } from 'lucide-react';

export default function LoginForm({ region, onLogin, onBack }) {
  const [username, setUsername] = useState('');

  const handleSubmit = (e) => {
    e.preventDefault();
    if (username.trim()) {
      // Generating a mock token for now
      const token = `mock-token-${Date.now()}`;
      onLogin(username, token);
    }
  };

  return (
    <div className='bg-white p-8 rounded-xl shadow-2xl max-w-md w-full'>
      <button onClick={onBack} className='text-gray-400 hover:text-gray-600 mb-6 flex items-center'>
        <ArrowLeft className='w-4 h-4 mr-1' /> Back
      </button>

      <div className='text-center mb-8'>
        <h2 className='text-2xl font-bold text-gray-800'>Join Chat</h2>
        <p className='text-gray-500 mt-2'>Connecting to {region.name}</p>
      </div>

      <form onSubmit={handleSubmit} className='space-y-6'>
        <div>
          <label className='block text-sm font-medium text-gray-700 mb-2'>Username</label>
          <div className='relative'>
            <User className='absolute left-3 top-3 w-5 h-5 text-gray-400' />
            <input
              type='text'
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className='w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none'
              placeholder='Enter your username'
              required
            />
          </div>
        </div>
        <button
          type='submit'
          className='w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 transition-colors font-medium'
        >
          Connect
        </button>
      </form>
    </div>
  );
}

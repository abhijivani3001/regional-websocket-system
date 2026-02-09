import React from 'react';
import { Globe, Server } from 'lucide-react';

const REGIONS = [
  { id: 'us-east', name: 'US East (N. Virginia)', port: 8080, flag: '🇺🇸' },
  { id: 'eu-west', name: 'EU West (Ireland)', port: 8081, flag: '🇪🇺' },
  { id: 'ap-south', name: 'Asia Pacific (Mumbai)', port: 8082, flag: '🇮🇳' },
];

export default function RegionSelector({ selectedRegion, onRegionSelect }) {
  return (
    <div className='bg-white p-8 rounded-xl shadow-2xl max-w-md w-full'>
      <div className='text-center mb-8'>
        <div className='bg-blue-100 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4'>
          <Globe className='w-8 h-8 text-blue-600' />
        </div>
        <h2 className='text-2xl font-bold text-gray-800'>Select Region</h2>
        <p className='text-gray-500 mt-2'>Choose the closest server for best performance</p>
      </div>

      <div className='space-y-4'>
        {REGIONS.map((region) => (
          <button
            key={region.id}
            onClick={() => onRegionSelect(region)}
            className={`w-full flex items-center p-4 rounded-lg border-2 transition-all ${
              selectedRegion?.id === region.id
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-blue-200 hover:bg-gray-50'
            }`}
          >
            <span className='text-2xl mr-4'>{region.flag}</span>
            <div className='text-left'>
              <div className='font-semibold text-gray-800'>{region.name}</div>
              <div className='text-xs text-gray-500 flex items-center mt-1'>
                <Server className='w-3 h-3 mr-1' /> Port: {region.port}
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

import type {NextConfig} from 'next';

const nextConfig: NextConfig = {
  /* config options here */
  typescript: {
    ignoreBuildErrors: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'placehold.co',
        port: '',
        pathname: '/**',
      },
      {
        protocol: 'https',
        hostname: 'images.unsplash.com',
        port: '',
        pathname: '/**',
      },
      {
        protocol: 'https',
        hostname: 'picsum.photos',
        port: '',
        pathname: '/**',
      },
    ],
  },
  // Performance optimizations
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
  },
  // Optimize imports
  experimental: {
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
  },
  // Enable static optimization where possible
  output: 'standalone',
  // Optimize bundle size
  webpack: (config, { isServer }) => {
    return config;
  },
  // Rewrite requests to the local backend to avoid Mixed Content issues
  async rewrites() {
    return [
      {
        source: '/api/proxy/status',
        destination: 'http://127.0.0.1:8080/status',
      },
      {
        source: '/api/proxy/start',
        destination: 'http://127.0.0.1:8080/start',
      },
      {
        source: '/api/proxy/stop',
        destination: 'http://127.0.0.1:8080/stop',
      },
      {
        source: '/api/proxy/api/:path*',
        destination: 'http://127.0.0.1:8080/api/:path*',
      },
    ];
  },
};

export default nextConfig;

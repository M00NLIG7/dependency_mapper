/** @type {import('next').NextConfig} */
const nextConfig = {
    webpack(config) {
        config.module.rules.push({
            test: /\.svg$/,
            use: ['@svgr/webpack']
        });
        return config;
    },
    output: 'export',
    distDir: 'out',
    ...(process.env.NODE_ENV === 'production' && {
        basePath: '/static',
        assetPrefix: '/static/',
    }),
    images: {
        unoptimized: true,
    },
};

export default nextConfig;

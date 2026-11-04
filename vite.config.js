// Private artwork is available to explicit web demo development only.
export default ({ command }) => ({ publicDir: command === 'serve' ? 'public' : false });

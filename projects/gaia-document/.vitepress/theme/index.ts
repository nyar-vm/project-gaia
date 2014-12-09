import DefaultTheme from 'vitepress/theme';
import enhanceApp from '../enhanceApp';

export default {
  extends: DefaultTheme,
  enhanceApp,
};
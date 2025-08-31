# SomeOtherModule is public, but SomeClass is not
# The test for SomeClass is namespaced under SomeOtherModule

module SomeModule
  module SomeOtherModule
    class SomeClass
      def bar; end
    end
  end
end

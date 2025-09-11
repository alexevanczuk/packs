
module SomeModule
  module SomeOtherModule
    RSpec.describe SomeClass do
      let(:my_var) { 42 }

      def helper_method
        "I'm a helper"
      end

      it "does something" do
        expect(true).to eq(true)
      end
    end
  end
end
